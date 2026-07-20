---
title: FastGPT知识库
slug: fastgpt知识库
summary: 不好用，别配
category: []
tags: []
status: published
updated: 2026-05-20T09:12
aliases: []
---
# 1. claude code工作内容

`CLAUDE.md`

```
# 知识库构建

MinerU（公式解析）+ FastGPT（知识库）+ DeepSeek（问答）

利用本目录下的pdf构建知识库

## 第一步：使用MinerU将PDF转为Markdown

使用 uv 创建并使用虚拟Python环境，利用pip安装使用mineru

## 第二步：由你对手动或创建Python脚本清洗Markdown

1. MinerU转markdown时自动对pdf分了十个窗口。这是按照页数分的，会导致章节内容连贯，识别所有输出的markdown内容。合理对markdown进行分节（按章节等）
2. 清洗掉无用的符号、页脚等（由你掌握具体清洗什么内容、如何清洗）
3. 清洗完、按章节分好的markdown重新输出到final文件夹

## 第三步：由用户手动在FastGPT上传markdown构建知识库
```

`run_mineru.sh`

```bash
#!/bin/bash
# Batch process PDF with MinerU
# Each batch processes 64 pages to avoid timeout

set -e

PDF="27张宇基础30讲高数.pdf"
OUTPUT="output"
BATCH_SIZE=64
TOTAL_PAGES=583

source .venv/bin/activate

echo "Starting MinerU batch processing: $TOTAL_PAGES pages in batches of $BATCH_SIZE"

START=0
BATCH=1
while [ $START -lt $TOTAL_PAGES ]; do
    END=$((START + BATCH_SIZE - 1))
    if [ $END -ge $TOTAL_PAGES ]; then
        END=$((TOTAL_PAGES - 1))
    fi

    BATCH_OUTPUT="${OUTPUT}/batch_$(printf '%02d' $BATCH)_pages_${START}_${END}"
    echo "======================================================"
    echo "Batch $BATCH: pages $START to $END ($((END - START + 1)) pages)"
    echo "Output: $BATCH_OUTPUT"
    echo "======================================================"

    mineru \
        -p "$PDF" \
        -o "$BATCH_OUTPUT" \
        --start "$START" \
        --end "$END" \
        -b hybrid-auto-engine \
        -f true \
        -t true

    echo "Batch $BATCH completed successfully."
    START=$((END + 1))
    BATCH=$((BATCH + 1))
done

echo "All batches completed!"
```

# 2. FastGPT构建

## 部署(docker)

```bash
# 这一条命令会自动为你拉取最新、且路径绝对正确的配置文件
bash <(curl -fsSL https://doc.fastgpt.cn/deploy/install.sh)
```

## Agent提示词

```
你是考研数学辅导老师，基于《张宇基础30讲》知识库为学生答疑解惑。

## 核心规则
严格依据知识库内容回答，不要凭记忆编造结论、公式或解题方法
如果知识库中没有相关内容，直接说"这个问题在基础30讲范围内没有涉及"，不要猜测
数学公式使用 LaTeX 格式输出，行内用 $...$，块级用 $$...$$

## 回答结构
概念类问题：先给出定义，再附上知识库中的关键注记或易错点
解题类问题：先点明考点（属于哪一讲），再给出解题思路和步骤
对比类问题：用表格或分点对比，让差异一目了然

## 风格
语言简洁，像老师在课堂上讲解一样，不啰嗦
善于用"注"来提醒常见错误和命题陷阱
如果问题涉及多个讲的知识点，主动说明关联关系

你可以根据实际使用效果微调，比如偏重解题就把解题规则放前面，偏重概念复习就强调定义和知识结构。
```