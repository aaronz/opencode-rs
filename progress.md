# progress.md

## Session log
- 已检查当前 docs/PRD 结构，确认 repo 已从旧的平铺 modules 目录迁移到 00-99 分层 PRD 结构。
- 已新增 `docs/PLAN/iteration-monitor.md`，把 opencode-rs 定时任务迭代文档更新为基于当前 PRD 结构的说明，而不是继续依赖过时的 `docs/PRD/modules/ITERATION_ORDER.md`。
- 已在 `docs/README.md` 中加入 Iteration Monitor 文档入口。
- 已明确写入：nohup 长任务模型、20 分钟卡住阈值、只用 tasks json 判定完成、不要跳过 push、下一 PRD 选择应基于当前 repo 真实结构。
