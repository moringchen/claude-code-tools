# karpahere

将可复用的 `/karpahere` slash 安装到你本地的 Claude Code skills 目录。

## 它会做什么

`/karpahere` 会向当前项目的 `.claude/CLAUDE.md` 追加一个带标记的 Karpathy 指南块。

- 安装到 `~/.claude/skills/karpahere/`
- 在本地携带 vendored 指南内容
- 如果目标文件已经包含这两个标记，则跳过重复插入
- 不依赖带版本号的 Claude plugin cache 路径

## Karpathy 思想

这里 vendored 进来的指南，针对的是 Andrej Karpathy 多次指出的几类 LLM 编码问题：静默做假设、把简单任务过度复杂化、顺手改动无关代码、以及在没有清晰成功标准时直接开工。

安装后的内容会把 Claude Code 往四个习惯上拉回去：

- 编码前先思考
- 优先选择真正满足需求的最简方案
- 只做与任务直接相关的精准修改
- 围绕可验证的成功标准推进执行

## 使用收益

在项目里使用 `/karpahere`，通常会带来这些好处：

- diff 里无关改动更少
- 过度设计和过度抽象更少
- 在写代码前更愿意先提澄清问题
- 任务输出更干净，更容易 review
- 实现结果和原始需求更一致

## 安装方法

```bash
curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/karpahere/install.sh | bash
```

## 安装后的文件

安装脚本会写入：

- `~/.claude/skills/karpahere/SKILL.md`
- `~/.claude/skills/karpahere/karpathy-guidelines.md`

## 工作方式

调用 `/karpahere` 时，已安装的 skill 会：

1. 以当前项目的 `.claude/CLAUDE.md` 为目标文件
2. 如果缺少 `.claude/` 或 `CLAUDE.md` 就自动创建
3. 写入前检查以下标记：
   - `<!-- karpahere:start -->`
   - `<!-- karpahere:end -->`
4. 只有当该标记块不存在时，才追加 vendored 的 Karpathy 指南内容

## 安装后验证

1. 启动一个新的 Claude Code 会话
2. 运行 `/help`
3. 确认可以看到 `/karpahere`
4. 在测试项目中调用 `/karpahere`，确认 `.claude/CLAUDE.md` 只会得到一个带标记的块

## 上游参考文档

这个工具还会把上游 Markdown 文件副本保存在 `upstream/` 下，方便和维护后的工具一起携带。

这些 vendored 副本来自迁移时本地已安装的 `andrej-karpathy-skills@karpathy-skills` 插件内容，导入时使用到的缓存快照版本为 `1.0.0`。

## 维护说明

仓库中的 `SKILL.md` 和 `karpathy-guidelines.md` 是受控源码；`install.sh` 内嵌了与之对应的内容，以保持 `curl -fsSL ... | bash` 安装方式完全自包含，而 `test_karpahere.zsh` 用来防止它们漂移。

## 许可证

MIT 许可证，与主项目一致。
