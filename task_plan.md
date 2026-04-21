# task_plan.md

## Goal
把 opencode-rs 的定时迭代任务补成可持续运行的自动流水线，包括严格的 modules 顺序、卡住检测、进展汇报、自动推进到下一个 module，以及定时 push。

## Phases
- [in_progress] Phase 1: 审视当前定时脚本与仓库状态
- [pending] Phase 2: 补自动推进下一个 module iteration
- [pending] Phase 3: 补定时 push 逻辑
- [pending] Phase 4: 验证 launchd 与脚本行为

## Notes
- 用户已明确不需要 verification-report 作为完成条件。
- 卡住判断基于仓库文件最近更新时间，阈值 20 分钟。
- 每次定时触发都要给用户发进展汇报。
- 需要把 modules 顺序真正用于自动执行，而不是只写文档。

## Errors Encountered
- 旧 LaunchAgent 指向 /tmp 脚本，已失效。
- repo 在 Documents 下，launchd 直接执行脚本会遇到 macOS 权限问题，已改为 ~/.openclaw/cron 入口。
