import matplotlib.pyplot as plt
import numpy as np

# 使用同样的数据
x = np.linspace(0, 10, 100)
y1 = np.sin(x) + 5
y2 = np.exp(x) / 100

# 1. 创建2行1列的子图，并共享X轴（共享后可以自动对齐且减少底部刻度标签）
fig, (ax1, ax2) = plt.subplots(nrows=1, ncols=2, figsize=(10, 8), sharex=True)

# 2. 在上方的子图 (ax1) 中绘制第一条线
ax1.plot(x, y1, color='tab:red')
ax1.set_ylabel('Y1 Axis (Sin(x))')
ax1.set_title('Sin(x) + 5') # 可以给每个子图单独设置标题
ax1.grid(True) # 添加网格线，更易读

# 3. 在下方的子图 (ax2) 中绘制第二条线
ax2.plot(x, y2, color='tab:blue')
ax2.set_xlabel('X Axis')    # 只需为最下面的子图设置X轴标签
ax2.set_ylabel('Y2 Axis (Exp(x))')
ax2.set_title('Exp(x) / 100')
ax2.grid(True)

# 4. 为整个图形添加一个总标题
fig.suptitle('Comparison using Subplots')

# 自动调整子图参数，避免重叠
plt.tight_layout()
plt.show()