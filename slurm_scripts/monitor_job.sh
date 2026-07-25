#!/bin/bash
#===========================================================
# 监控 methx 测试任务
#===========================================================

JOB_ID=$1

if [ -z "$JOB_ID" ]; then
    # 使用最新的任务
    JOB_ID=$(cat .last_job_id 2>/dev/null | head -1 | awk '{print $2}')
    if [ -z "$JOB_ID" ]; then
        echo "用法: $0 <JOB_ID>"
        exit 1
    fi
fi

echo "=========================================="
echo "监控任务: $JOB_ID"
echo "=========================================="
echo ""

# 检查任务是否还在运行
while squeue -j $JOB_ID &> /dev/null; do
    clear
    echo "=========================================="
    echo "任务监控: $JOB_ID"
    echo "=========================================="
    echo "时间: $(date)"
    echo ""

    # 显示任务状态
    echo "任务状态:"
    squeue -j $JOB_ID
    echo ""

    # 显示最后20行输出
    echo "=== 输出日志 (最后20行) ==="
    if [ -f "logs/methx_quick_${JOB_ID}.out" ]; then
        tail -20 "logs/methx_quick_${JOB_ID}.out"
    elif [ -f "logs/methx_test_${JOB_ID}.out" ]; then
        tail -20 "logs/methx_test_${JOB_ID}.out"
    else
        echo "日志文件尚未创建"
    fi

    echo ""
    echo "按 Ctrl+C 退出监控 (任务继续运行)"
    echo ""

    sleep 5
done

clear
echo "=========================================="
echo "任务已完成: $JOB_ID"
echo "=========================================="
echo ""

# 显示最终状态
echo "最终状态:"
sacct -j $JOB_ID --format=JobID,JobName,State,ExitCode,Elapsed,AllocCPUS,MaxRSS
echo ""

# 显示完整输出
echo "=== 完整输出日志 ==="
if [ -f "logs/methx_quick_${JOB_ID}.out" ]; then
    cat "logs/methx_quick_${JOB_ID}.out"
elif [ -f "logs/methx_test_${JOB_ID}.out" ]; then
    cat "logs/methx_test_${JOB_ID}.out"
fi

# 显示错误（如果有）
echo ""
echo "=== 错误日志 ==="
if [ -f "logs/methx_quick_${JOB_ID}.err" ]; then
    cat "logs/methx_quick_${JOB_ID}.err"
elif [ -f "logs/methx_test_${JOB_ID}.err" ]; then
    cat "logs/methx_test_${JOB_ID}.err"
fi

echo ""
echo "=========================================="
