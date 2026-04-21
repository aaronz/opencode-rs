#!/usr/bin/env python3
"""
SpecKit 迭代开发 v3.0 - Python 实现
基于 iterate-prd.sh 重构
"""

import argparse
import datetime
import json
import os
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

DEFAULT_MODEL = "minimax-cn/MiniMax-M2.7"
OPENCODE_CMD = shutil.which("opencode") or os.path.expanduser("~/.opencode/bin/opencode")


@dataclass
class Config:
    workspace_dir: Path
    model: str = DEFAULT_MODEL
    max_implementation_rounds: int = 10
    prd_input: Optional[str] = None
    log_file: Optional[Path] = None
    verbose: bool = False
    resume_iteration: Optional[int] = None


def ts_echo(msg: str, config: Config):
    """带时间戳输出"""
    timestamp = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    formatted = f"[{{timestamp}}] {{msg}}"
    print(formatted)
    if config.log_file:
        with open(config.log_file, "a", encoding="utf-8") as f:
            f.write(formatted + "\n")


def parse_args() -> Config:
    parser = argparse.ArgumentParser(description="SpecKit 迭代开发")
    parser.add_argument("--resume", type=int, help="恢复迭代编号")
    parser.add_argument("--model", "-m", default=DEFAULT_MODEL, help="模型名称")
    parser.add_argument("--rounds", "-r", type=int, default=10, help="最大实现轮次")
    parser.add_argument("--prd", "-p", help="PRD 文件或目录")
    parser.add_argument("--log", help="日志文件路径")
    parser.add_argument("--verbose", "-v", action="store_true", help="详细输出")
    parser.add_argument("--workspace", "-w", default=".", help="工作目录")

    args = parser.parse_args()

    workspace_dir = Path(args.workspace).resolve()
    config = Config(
        workspace_dir=workspace_dir,
        model=args.model,
        max_implementation_rounds=args.rounds,
        prd_input=args.prd,
        log_file=Path(args.log) if args.log else None,
        verbose=args.verbose,
        resume_iteration=args.resume,
    )
    return config


def get_next_iteration(workspace_dir: Path, resume: Optional[int]) -> int:
    """获取下一个迭代编号"""
    if resume:
        iteration_dir = workspace_dir / "iterations" / f"iteration-{{resume}}"
        if not iteration_dir.exists():
            print(f"❌ 指定迭代不存在: {{iteration_dir}}")
            sys.exit(1)
        return resume

    iterations_dir = workspace_dir / "iterations"
    if not iterations_dir.exists():
        return 1

    existing = list(iterations_dir.iterdir())
    if not existing:
        return 1

    numbers = []
    for d in existing:
        match = re.match(r"iteration-(\d+)", d.name)
        if match:
            numbers.append(int(match.group(1)))

    return max(numbers) + 1 if numbers else 1


def resolve_prd(config: Config) -> tuple[Path, Path]:
    """解析 PRD 输入，返回输出目录和 PRD 文件路径"""
    outputs_dir = config.workspace_dir / "iterations"

    if config.prd_input:
        prd_input = Path(config.prd_input)
        if not prd_input.exists():
            print(f"❌ PRD路径不存在: {{config.prd_input}}")
            sys.exit(1)

        if prd_input.is_dir():
            md_files = sorted(prd_input.glob("*.md"))
            if not md_files:
                print(f"❌ 文件夹中未找到.md文件: {{config.prd_input}}")
                sys.exit(1)

            prd_path = outputs_dir / "_prd_combined.md"
            with open(prd_path, "w", encoding="utf-8") as out:
                for f in md_files:
                    with open(f, encoding="utf-8") as inp:
                        out.write(inp.read())
            ts_echo(f"📂 使用PRD文件夹: {{config.prd_input}} (合并{{len(md_files)}}个文件)", config)
            return outputs_dir, prd_path
        else:
            ts_echo(f"📄 使用PRD文件: {{prd_input}}", config)
            return outputs_dir, prd_input
    else:
        prd_path = config.workspace_dir / "PRD.md"
        if not prd_path.exists():
            print(f"❌ PRD文件不存在: {{prd_path}}")
            sys.exit(1)
        return outputs_dir, prd_path


def run_opencode(prompt: str, config: Config, export_file: Optional[Path] = None) -> tuple[int, str]:
    """运行 opencode 并返回退出码和输出"""
    start_time = datetime.datetime.now()
    ts_echo(f"[DEBUG] run_opencode 开始 | model={{config.model}}", config)

    cmd = [
        OPENCODE_CMD, "run",
        "-m", config.model,
        "--dangerously-skip-permissions",
        "--format", "json",
        "--",
        prompt
    ]

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            cwd=config.workspace_dir,
        )
        output = result.stdout + result.stderr
        elapsed = (datetime.datetime.now() - start_time).total_seconds()
        ts_echo(f"[DEBUG] opencode 完成 | elapsed={{elapsed:.1f}}s | exit_code={{result.returncode}}", config)

        if export_file:
            export_file.parent.mkdir(parents=True, exist_ok=True)
            with open(export_file, "w", encoding="utf-8") as f:
                f.write(output)
            ts_echo(f"[DEBUG] 已导出到: {{export_file}}", config)

        if result.returncode != 0:
            ts_echo(f"⚠️  opencode 异常退出: exit_code={{result.returncode}}", config)

        return result.returncode, output

    except Exception as e:
        ts_echo(f"❌ opencode 执行失败: {{e}}", config)
        return 1, str(e)


def check_file_exists(path: Path, min_size: int = 10) -> bool:
    return path.exists() and path.stat().st_size >= min_size


def save_checkpoint(iteration: int, phase: str, outputs_dir: Path, config: Config):
    checkpoint_file = outputs_dir / ".checkpoint"
    with open(checkfile_file, "w") as f:
        f.write(f"iteration={{iteration}}\n")
        f.write(f"phase={{phase}}\n")
        f.write(f"timestamp={{int(datetime.datetime.now().timestamp())}}\n")
    ts_echo(f"checkpoint: iteration={{iteration}}, phase={{phase}}", config)


NON_INTERACTIVE_CONSTRAINT = """
## 重要约束
- 禁止使用 subagent 或 task 工具 spawning 其他 agent
- 禁止将工作委托给其他 agent
- 必须直接在当前 session 中完成所有分析工作
- 只使用 Read、Write、Edit、Grep、LSP 等直接工具
- 禁止向用户提问或请求确认，必须独立做出最佳判断并继续执行
- 如遇不确定情况，使用你自己的最佳判断，无需等待用户输入"""


def run_phase_gap_analysis(prd_path: Path, outputs_dir: Path, config: Config) -> Path:
    """Phase 1: Gap Analysis"""
    gap_file = outputs_dir / "gap-analysis.md"

    if check_file_exists(gap_file):
        ts_echo("⏭️  跳过Gap Analysis（已存在）", config)
        return gap_file

    ts_echo("📝 生成 Gap Analysis...", config)

    with open(prd_path, encoding="utf-8") as f:
        prd_content = f.read()

    prompt = f"""分析当前实现与PRD的差距，并将完整的差距分析报告写入文件：{{gap_file}}

{{NON_INTERACTIVE_CONSTRAINT}}

## PRD
{{prd_content}}

## 任务
1. 读取当前实现目录结构（src/目录、iterations/src/目录等）
2. 基于上述PRD识别核心功能需求
3. 对比实现与PRD的差距

## 分析维度
1. 功能完整性：PRD中描述的功能是否都已实现？
2. 接口完整性：API是否完整？CRUD是否齐全？
3. 前端完整性：PRD中描述的页面/组件是否都已实现？
4. 数据模型：PRD中的数据实体是否都已建模？
5. 配置管理：PRD中要求的配置项是否都已实现？
6. 测试覆盖：是否有必要的测试？

## 通用差距识别
- 缺失的功能模块
- 不完整的实现
- 未连接的模块
- 硬编码/魔法数字
- 错误处理缺失
- 类型定义缺失

## 输出要求
将完整的差距分析报告写入到：{{gap_file}}

报告必须包含：
1. 差距列表（表格格式：差距项 | 严重程度 | 模块 | 修复建议）
2. P0/P1/P2问题分类（必须包含P0阻断性问题）
3. 技术债务清单
4. 实现进度总结"""

    export_file = config.workspace_dir / "sessions" / f"iteration-{{config.resume_iteration}}" / "gap_analysis.json"
    run_opencode(prompt, config, export_file)

    if check_file_exists(gap_file):
        ts_echo("✅ Gap Analysis 完成", config)
    else:
        ts_echo("⚠️  Gap Analysis 生成失败", config)

    return gap_file


def run_phase_spec(prd_path: Path, gap_analysis: Path, outputs_dir: Path, iteration: int, config: Config) -> Path:
    """Phase 2: Update Spec"""
    spec_file = outputs_dir / f"spec_v{{iteration}}.md"

    if check_file_exists(spec_file):
        ts_echo("⏭️  跳过Spec更新（已存在）", config)
        return spec_file

    ts_echo("📝 生成 Spec...", config)

    with open(prd_path, encoding="utf-8") as f:
        prd_content = f.read()

    with open(gap_analysis, encoding="utf-8") as f:
        gap_content = f.read()

    prompt = f"""基于PRD和差距分析，更新规格文档，并写入文件：{{spec_file}}

{{NON_INTERACTIVE_CONSTRAINT}}

## PRD
{{prd_content}}

## 差距分析
{{gap_content}}

## 任务
1. 基于差距分析，更新spec.md
2. 确保新功能有对应的规格定义
3. 添加功能需求编号(FR-XXX)

## 输出要求
将更新后的规格文档写入到：{{spec_file}}"""

    export_file = config.workspace_dir / "sessions" / f"iteration-{{config.resume_iteration}}" / "spec.json"
    run_opencode(prompt, config, export_file)

    if check_file_exists(spec_file):
        ts_echo("✅ Spec 更新完成", config)
    else:
        ts_echo("⚠️  Spec 生成失败", config)

    return spec_file


def run_phase_plan(spec_file: Path, gap_analysis: Path, outputs_dir: Path, iteration: int, config: Config) -> tuple[Path, Path]:
    """Phase 3: Update Plan and Tasks"""
    plan_file = outputs_dir / f"plan_v{{iteration}}.md"
    tasks_file = outputs_dir / f"tasks_v{{iteration}}.md"
    tasks_json = outputs_dir / f"tasks_v{{iteration}}.json"

    if check_file_exists(plan_file) and check_file_exists(tasks_file):
        ts_echo("⏭️  跳过Plan/Tasks更新（已存在）", config)
        return tasks_file, tasks_json

    ts_echo("📝 生成 Plan 和 Tasks...", config)

    with open(spec_file, encoding="utf-8") as f:
        spec_content = f.read()

    with open(gap_analysis, encoding="utf-8") as f:
        gap_content = f.read()

    prompt = f"""基于Spec更新实现计划和任务清单，并将它们写入文件。

{{NON_INTERACTIVE_CONSTRAINT}}

## Spec
{{spec_content}}

## 差距分析
{{gap_content}}

## 任务
1. 更新实现计划
2. 更新任务清单
3. 确保P0任务优先

## 输出要求
将更新后的计划写入到：{{plan_file}}
将更新后的任务清单写入到：{{tasks_file}}"""

    export_file = config.workspace_dir / "sessions" / f"iteration-{{config.resume_iteration}}" / "plan.json"
    run_opencode(prompt, config, export_file)

    if check_file_exists(tasks_file):
        ts_echo("✅ Tasks 生成完成", config)
        generate_tasks_json(tasks_file, tasks_json, config)
    else:
        ts_echo("⚠️  Tasks 生成失败", config)

    return tasks_file, tasks_json


def generate_tasks_json(tasks_file: Path, tasks_json: Path, config: Config):
    """生成结构化任务 JSON"""
    if check_file_exists(tasks_json):
        ts_echo(f"⏭️  跳过JSON生成（已存在）: {{tasks_json}}", config)
        return

    ts_echo(f"📝 生成结构化任务JSON: {{tasks_json}}", config)

    with open(tasks_file, encoding="utf-8") as f:
        tasks_content = f.read()

    prompt = f"""基于任务Markdown文件，生成一个结构化的JSON任务文件。

## 重要约束
- 禁止使用 subagent 或 task 工具 spawning 其他 agent
- 必须直接在当前 session 中完成

## 任务Markdown文件
{{tasks_content}}

## 输出要求
将JSON写入到文件：{{tasks_json}}

JSON格式必须包含以下字段：
{{{{
  "tasks": [
    {{{{
      "id": "任务ID（如T-001）",
      "priority": "P0|P1|P2",
      "title": "任务简短标题",
      "description": "任务详细描述",
      "status": "todo|done|in_progress",
      "test_criteria": ["测试标准1", "测试标准2"],
      "test_commands": ["cargo test --package <pkg>", "npm run build"],
      "impl_notes": "实现注意事项",
      "dependencies": ["依赖的任务ID"]
    }}}}
  ]
}}

## 要求
1. 每个任务必须有清晰可测试的test_criteria
2. test_commands必须是可自动化执行的验证命令
3. parse Markdown status markers (- [ ] = todo, - [x] = done)
4. output must be valid JSON, write directly to file with no other content"""

    export_file = config.workspace_dir / "sessions" / f"iteration-{{config.resume_iteration}}" / "tasks_json.json"
    run_opencode(prompt, config, export_file)

    if check_file_exists(tasks_json):
        ts_echo("✅ JSON生成成功", config)
    else:
        ts_echo("⚠️  JSON生成失败，将使用 fallback 解析", config)
        generate_tasks_json_fallback(tasks_file, tasks_json, config)


def generate_tasks_json_fallback(tasks_file: Path, tasks_json: Path, config: Config):
    """Fallback: 从 Markdown 解析任务"""
    import re

    with open(tasks_file, encoding="utf-8") as f:
        content = f.read()

    tasks = []
    current_priority = None
    current_task = None

    lines = content.split("\n")
    for line in lines:
        priority_match = re.match(r"^##\s+(P0|P1|P2)", line)
        if priority_match:
            if current_task:
                tasks.append(current_task)
            current_priority = priority_match.group(1)
            current_task = None
            continue

        task_match = re.match(r"^###\s+([A-Z]+-\d+):\s*(.+)", line)
        if task_match and current_priority:
            if current_task:
                tasks.append(current_task)
            current_task = {{
                "id": task_match.group(1),
                "priority": current_priority,
                "title": task_match.group(2),
                "description": "",
                "status": "todo",
                "test_criteria": ["代码编译通过", "功能测试通过"],
                "test_commands": ["cargo build"],
                "impl_notes": "",
                "dependencies": []
            }}
            continue

        if current_task and line.strip():
            current_task["description"] += line.strip() + "\n"

    if current_task:
        tasks.append(current_task)

    with open(tasks_json, "w", encoding="utf-8") as f:
        json.dump({{"tasks": tasks}}, f, indent=2, ensure_ascii=False)

    ts_echo(f"✅ Fallback JSON生成完成: {{len(tasks)}} tasks", config)


def load_tasks(tasks_json: Path) -> list[dict]:
    """加载任务列表"""
    if not tasks_json.exists():
        return []

    with open(tasks_json, encoding="utf-8") as f:
        data = json.load(f)

    if isinstance(data, list):
        return data
    return data.get("tasks", [])


def save_tasks(tasks_json: Path, tasks: list[dict]):
    """保存任务列表"""
    with open(tasks_json, "w", encoding="utf-8") as f:
        json.dump({{"tasks": tasks}}, f, indent=2, ensure_ascii=False)


def get_next_todo_task(tasks: list[dict]) -> Optional[dict]:
    """获取下一个待办任务"""
    for task in tasks:
        if task.get("status") == "todo":
            return task
    return None


def count_todo_tasks(tasks: list[dict]) -> int:
    return sum(1 for t in tasks if t.get("status") == "todo")


def count_done_tasks(tasks: list[dict]) -> int:
    return sum(1 for t in tasks if t.get("status") == "done")


def count_remaining_p0_p1(tasks: list[dict]) -> int:
    return sum(1 for t in tasks
               if t.get("priority") in ["P0", "P1"]
               and t.get("status") != "done")


def update_task_status(tasks: list[dict], task_id: str, status: str) -> list[dict]:
    """更新任务状态"""
    for task in tasks:
        if task.get("id") == task_id:
            task["status"] = status
            break
    return tasks


def run_phase_implementation(tasks_json: Path, spec_file: Path, outputs_dir: Path, max_rounds: int, config: Config):
    """Phase 4: Per-Task Implementation Loop"""
    tasks = load_tasks(tasks_json)

    if not tasks:
        ts_echo("⚠️  没有任务可执行", config)
        return

    for round_num in range(1, max_rounds + 1):
        ts_echo("", config)
        ts_echo(f"==============================================", config)
        ts_echo(f"外循环轮次 {{round_num}}/{{max_rounds}}", config)
        ts_echo(f"==============================================", config)

        remaining = count_remaining_p0_p1(tasks)
        ts_echo(f"剩余未完成的P0/P1任务: {{remaining}}", config)

        todo_count = count_todo_tasks(tasks)
        done_count = count_done_tasks(tasks)
        total = todo_count + done_count
        ts_echo(f"任务进度: {{done_count}}/{{total}} 完成", config)

        if remaining == 0 and todo_count == 0:
            ts_echo("所有P0/P1任务已完成!", config)
            break

        if todo_count == 0:
            ts_echo("没有待办任务了", config)
            break

        ts_echo("开始逐个实现待办任务...", config)

        while True:
            task = get_next_todo_task(tasks)
            if not task:
                ts_echo("所有待办任务已处理完毕", config)
                break

            implement_task(task, tasks, tasks_json, spec_file, config)
            save_tasks(tasks_json, tasks)

            remaining = count_remaining_p0_p1(tasks)
            if remaining == 0:
                ts_echo("", config)
                ts_echo("🎉 所有P0/P1阻断性问题已解决!", config)
                break

        if round_num == max_rounds:
            ts_echo(f"达到最大外循环轮次", config)
            remaining = count_remaining_p0_p1(tasks)
            if remaining > 0:
                ts_echo(f"仍有 {{remaining}} 个P0/P1任务未完成", config)
            ts_echo("继续到验证阶段...", config)
            break


def implement_task(task: dict, tasks: list[dict], tasks_json: Path, spec_file: Path, config: Config):
    """实现单个任务"""
    task_id = task.get("id", "unknown")

    ts_echo("", config)
    ts_echo("----------------------------------------------", config)
    ts_echo(f"🎯 实现任务: {{task_id}}", config)
    ts_echo("----------------------------------------------", config)

    ts_echo(f"任务详情: 标题={{task.get('title')}}, 优先级={{task.get('priority')}}", config)

    tasks = update_task_status(tasks, task_id, "in_progress")
    save_tasks(tasks_json, tasks)

    ts_echo("", config)
    ts_echo("开始实现...", config)

    with open(spec_file, encoding="utf-8") as f:
        spec_content = f.read()

    test_commands = task.get("test_commands", ["cargo build"])
    test_criteria = task.get("test_criteria", [])

    prompt = f"""实现任务：{{task_id}}

{{NON_INTERACTIVE_CONSTRAINT}}

## 任务信息
ID: {{task_id}}
标题: {{task.get('title')}}
描述: {{task.get('description')}}
优先级: {{task.get('priority')}}
测试标准: {{chr(10).join(test_criteria)}}
测试命令: {{chr(10).join(test_commands)}}
实现注意事项: {{task.get('impl_notes', '')}}
依赖: {{task.get('dependencies', [])}}

## Spec
{{spec_content}}

## 实现目录
./iterations/src/

## 任务
1. 分析任务需求和测试标准
2. 实现代码
3. 运行测试命令验证
4. 确保 cargo build 和 cargo test 通过
5. 完成后更新任务状态

## 验证
- 必须通过: cargo build
- 必须通过: cargo test

## 完成后的操作
1. 更新任务JSON文件中的状态为 done
2. 如果有对应的Markdown任务文件，也需要更新状态为 ✅ Done
3. 提交代码变更"""

    export_file = config.workspace_dir / "sessions" / f"iteration-{{config.resume_iteration}}" / f"task_{{task_id}}.json"
    run_opencode(prompt, config, export_file)

    ts_echo("", config)
    ts_echo("验证实现...", config)

    test_passed = True
    test_output = ""

    for cmd in test_commands:
        ts_echo(f"执行: {{cmd}}", config)
        try:
            result = subprocess.run(
                cmd,
                shell=True,
                capture_output=True,
                text=True,
                cwd=config.workspace_dir / "opencode-rust",
                timeout=300,
            )
            test_output = result.stdout + result.stderr
            if result.returncode != 0:
                ts_echo("⚠️  测试有问题，请检查", config)
                test_passed = False
                break
        except subprocess.TimeoutExpired:
            ts_echo("⚠️  测试超时", config)
            test_passed = False
            break
        except Exception as e:
            ts_echo(f"⚠️  测试执行失败: {{e}}", config)
            test_passed = False
            break

    if test_passed:
        ts_echo("测试通过", config)

    if not test_passed:
        ts_echo("", config)
        ts_echo("❌ 测试失败，重新生成修复方案...", config)

        fix_prompt = f"""任务 {{task_id}} 测试失败，需要修复。

## 测试失败输出
```
{{test_output}}
```

## 任务信息
ID: {{task_id}}
标题: {{task.get('title')}}
描述: {{task.get('description')}}
测试命令: {{chr(10).join(test_commands)}}

## Spec
{{spec_content}}

## 重要约束
- 禁止使用 subagent 或 task 工具 spawning 其他 agent
- 必须直接在当前 session 中修复所有问题
- 分析测试失败原因，修复代码，确保所有测试通过

## 任务
1. 分析测试失败原因
2. 修复代码问题
3. 重新运行测试验证修复
4. 确保 cargo build 和 cargo test 通过

## 验证
- 必须通过: cargo build
- 必须通过: cargo test

## 完成后的操作
1. 更新任务JSON文件中的状态为 done
2. 如果有对应的Markdown任务文件，也需要更新状态为 ✅ Done
3. 提交代码变更"""

        export_file = config.workspace_dir / "sessions" / f"iteration-{{config.resume_iteration}}" / f"task_{{task_id}}_fix.json"
        run_opencode(fix_prompt, config, export_file)

        ts_echo("", config)
        ts_echo("验证修复...", config)

        test_passed = True
        for cmd in test_commands:
            result = subprocess.run(
                cmd,
                shell=True,
                capture_output=True,
                text=True,
                cwd=config.workspace_dir / "opencode-rust",
                timeout=300,
            )
            if result.returncode != 0:
                test_passed = False
                break

        if not test_passed:
            ts_echo("⚠️  再次测试失败，标记为需手动检查，继续处理下一任务", config)
            tasks = update_task_status(tasks, task_id, "manual_check")
            return

    tasks = update_task_status(tasks, task_id, "done")
    save_tasks(tasks_json, tasks)

    tasks_file = tasks_json.with_suffix(".md")
    if tasks_file.exists():
        content = tasks_file.read_text(encoding="utf-8")
        content = re.sub(
            rf"^### {{re.escape(task_id)}}:.*",
            f"### {{task_id}}: ✅ Done",
            content,
            flags=re.MULTILINE,
        )
        tasks_file.write_text(content, encoding="utf-8")

    try:
        result = subprocess.run(
            ["git", "status", "--porcelain"],
            capture_output=True,
            text=True,
            cwd=config.workspace_dir,
        )
        if result.stdout.strip():
            subprocess.run(["git", "add", "-A"], cwd=config.workspace_dir)
            commit_msg = f"impl({{task_id}}): {{task.get('title', task_id)[:50]}}"
            subprocess.run(["git", "commit", "-m", commit_msg], cwd=config.workspace_dir)
            ts_echo("提交完成", config)
    except Exception as e:
        ts_echo(f"⚠️  提交失败: {{e}}", config)

    ts_echo("", config)
    ts_echo(f"任务 {{task_id}} 完成", config)


def main():
    config = parse_args()

    ts_echo("==============================================", config)
    ts_echo("SpecKit 迭代开发 v3.0 (Python版)", config)
    ts_echo("==============================================", config)
    ts_echo(f"工作目录: {{config.workspace_dir}}", config)
    if config.log_file:
        ts_echo(f"日志文件: {{config.log_file}}", config)

    outputs_dir, prd_path = resolve_prd(config)

    iteration = get_next_iteration(config.workspace_dir, config.resume_iteration)
    config.resume_iteration = iteration
    outputs_dir = outputs_dir / f"iteration-{{iteration}}"

    if not config.resume_iteration:
        outputs_dir.mkdir(parents=True, exist_ok=True)

    session_export_dir = config.workspace_dir / "sessions" / f"iteration-{{iteration}}"
    session_export_dir.mkdir(parents=True, exist_ok=True)

    if not config.log_file:
        config.log_file = config.workspace_dir / "sessions" / f"iteration-{{iteration}}_{{datetime.datetime.now().strftime('%Y%m%d_%H%M%S')}}.log"

    ts_echo(f"迭代目录: {{outputs_dir}}", config)
    ts_echo(f"模型: {{config.model}}", config)
    ts_echo(f"最大外循环轮次: {{config.max_implementation_rounds}}", config)
    ts_echo("", config)

    ts_echo("[1/4] 执行PRD差距分析...", config)
    save_checkpoint(iteration, "phase1", outputs_dir, config)
    gap_analysis = run_phase_gap_analysis(prd_path, outputs_dir, config)

    ts_echo("", config)
    ts_echo("[2/4] 更新Spec...", config)
    save_checkpoint(iteration, "phase2", outputs_dir, config)
    spec_file = run_phase_spec(prd_path, gap_analysis, outputs_dir, iteration, config)

    ts_echo("", config)
    ts_echo("[3/4] 更新Plan和Tasks...", config)
    save_checkpoint(iteration, "phase3", outputs_dir, config)
    tasks_file, tasks_json = run_phase_plan(spec_file, gap_analysis, outputs_dir, iteration, config)

    ts_echo("", config)
    ts_echo("[4/4] Per-Task 实现循环...", config)
    save_checkpoint(iteration, "phase4", outputs_dir, config)
    run_phase_implementation(tasks_json, spec_file, outputs_dir, config.max_implementation_rounds, config)

    ts_echo("", config)
    ts_echo("==============================================", config)
    ts_echo("SpecKit 迭代完成!", config)
    ts_echo("==============================================", config)
    ts_echo(f"迭代目录: {{outputs_dir}}", config)
    ts_echo(f"任务文件: {{tasks_json}}", config)
    ts_echo(f"日志保存于: {{config.log_file}}", config)


if __name__ == "__main__":
    main()
