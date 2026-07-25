#!/bin/bash
#===========================================================
# methx 测试任务提交助手
#===========================================================

set -e

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=============================================="
echo "   methx 测试任务提交助手"
echo -e "==============================================${NC}\n"

# 检查是否在正确的目录
if [ ! -f "target/release/methx" ]; then
    echo -e "${YELLOW}警告: 未找到编译的二进制文件${NC}"
    echo "请先运行: cargo build --release"
    echo ""
    read -p "是否继续? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# 显示测试选项
echo "请选择测试类型:"
echo "  1) 快速测试 (2个样本, ~30分钟, 8核32GB)"
echo "  2) 完整测试 (12个样本, ~2小时, 16核64GB)"
echo "  3) 自定义配置"
echo ""
read -p "请输入选项 [1-3]: " choice

case $choice in
    1)
        SCRIPT="run_quick_test.sbatch"
        echo -e "\n${GREEN}已选择: 快速测试${NC}"
        echo "脚本: $SCRIPT"
        ;;
    2)
        SCRIPT="run_test.sbatch"
        echo -e "\n${GREEN}已选择: 完整测试${NC}"
        echo "脚本: $SCRIPT"
        ;;
    3)
        echo -e "\n${YELLOW}自定义配置${NC}"
        read -p "CPU 核心数 [8]: " cpus
        cpus=${cpus:-8}
        read -p "内存 (GB) [32]: " mem
        mem=${mem:-32}
        read -p "时间限制 (分钟) [30]: " time
        time=${time:-30}
        read -p "样本数量 [2]: " samples
        samples=${samples:-2}

        # 创建自定义脚本
        SCRIPT="run_custom_test_${SLURM_JOB_ID:-$$}.sbatch"
        cp run_quick_test.sbatch "$SCRIPT"

        # 更新资源配置
        sed -i "s/--cpus-per-task=8/--cpus-per-task=$cpus/" "$SCRIPT"
        sed -i "s/--mem=32G/--mem=${mem}G/" "$SCRIPT"
        sed -i "s/--time=00:30:00/--time=${time}:00/" "$SCRIPT"

        echo -e "${GREEN}自定义脚本已创建: $SCRIPT${NC}"
        ;;
    *)
        echo -e "${YELLOW}无效选项，使用默认快速测试${NC}"
        SCRIPT="run_quick_test.sbatch"
        ;;
esac

# 确认提交
echo ""
echo "配置摘要:"
echo "  脚本: $SCRIPT"
echo "  当前目录: $(pwd)"
echo ""

read -p "确认提交任务? (y/n) " -n 1 -r
echo

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "已取消"
    exit 0
fi

# 创建日志目录
mkdir -p logs

# 提交任务
echo -e "${BLUE}=============================================${NC}"
echo "正在提交任务..."
echo -e "${BLUE}=============================================${NC}"

JOB_ID=$(sbatch "$SCRIPT" | awk '{print $4}')

if [ -z "$JOB_ID" ]; then
    echo -e "${YELLOW}任务提交失败${NC}"
    echo "请检查 SLURM 配置"
    exit 1
fi

echo -e "${GREEN}任务已提交!${NC}"
echo ""
echo "任务信息:"
echo "  Job ID: $JOB_ID"
echo "  脚本: $SCRIPT"
echo ""
echo "查看状态:"
echo "  squeue -j $JOB_ID"
echo ""
echo "查看日志:"
echo "  tail -f logs/methx_*_${JOB_ID}.out"
echo ""

# 等待几秒后检查任务状态
sleep 2

if squeue -j $JOB_ID &> /dev/null; then
    echo -e "${GREEN}任务正在运行...${NC}"
    echo ""
    echo "实时监控命令:"
    echo "  watch -n 5 squeue -j $JOB_ID"
else
    echo -e "${YELLOW}任务已完成或已退出${NC}"
    echo ""
    echo "查看日志:"
    echo "  cat logs/methx_*_${JOB_ID}.out"
fi

echo ""
echo -e "${BLUE}=============================================${NC}"
echo "任务提交完成!"
echo -e "${BLUE}=============================================${NC}"
echo ""

# 保存任务信息
echo "Job ID: $JOB_ID" > .last_job_id
echo "Script: $SCRIPT" >> .last_job_id
echo "Time: $(date)" >> .last_job_id

echo "任务信息已保存到: .last_job_id"
echo ""
