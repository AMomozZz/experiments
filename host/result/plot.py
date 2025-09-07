import math
import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

# 读取三个 CSV 文件并合并
df1 = pd.read_csv("./host/result/e1_e3_e5/experiment_results_usedonly.csv")
df2 = pd.read_csv("./host/result/e1_e3_e5/experiment_results_usedonly_opt.csv")
df3 = pd.read_csv("./host/result/e1_e3_e5/experiment_results.csv")
dfs = {'experiment_results_usedonly': df1, 'experiment_results_usedonly_opt': df2, 'experiment_results': df3}
file_names_dict = {'experiment_results': 'all_func', 'experiment_results_usedonly': 'usedonly_func', 'experiment_results_usedonly_opt': 'usedonly_func_size_opt'}
file_names = [[('experiment_results', 'all_func'), ('experiment_results_usedonly', 'usedonly_func'), ('experiment_results_usedonly_opt', 'usedonly_func_size_opt')], [('bidComponent100_usedonly_opt', 'usedonly_func_size_opt'), ('bidComponent100_usedonly', 'usedonly_func'), ('bidComponent100', 'all_func'), ('bidComponent10000_usedonly_opt', 'usedonly_func_size_opt'), ('bidComponent10000_usedonly', 'usedonly_func'), ('bidComponent10000', 'all_func')]]
alldf = pd.concat([df1, df2, df3], ignore_index=True)

# # 提取 e1 和 e3 的数据
# e1_data = df[df['experiment'] == 'e1']
# e3_data = df[df['experiment'] == 'e3']

# 定义要展示的方法名称（按你提供的顺序）
methods = [
    "io",
    "native opt",
    "wasm (pass all data)",
    "wasm opt (pruned data)",
    "wasm opt2 (pruned data + filter conditions in wasm)",
    "wasm opt3 (structured data + filter conditions in wasm + directly returns a not pruned data)",
    "wasm opt4 (structured data + filter conditions in wasm + directly returns a pruned data)"
]

# 定义数据规模
sizes = [100, 1000, 10000, 100000, 1000000]

# 为 e1 和 e3 分别准备数据
def prepare_plot_data(onedata:pd.DataFrame, alldata:pd.DataFrame, experiment, methods, sizes):
    plot_data = {}
    for method in methods:
        method_data = []
        for size in sizes:
            # 获取对应规模和方法的 no_warmup_avg 值，取平均值（若有重复）
            if method in ["io", "native opt"]:
                values = alldata[(alldata['name'] == method) & (alldata['size'] == size) & (alldata['experiment'] == experiment)]['no_warmup_avg']
            else:
                values = onedata[(onedata['name'] == method) & (onedata['size'] == size) & (onedata['experiment'] == experiment)]['no_warmup_avg']
                if len(values) > 1: raise ValueError
            if len(values) > 0:
                if method == 'native opt':
                    method_data.append(values.max())
                else:
                    method_data.append(values.min())
            else:
                method_data.append(np.nan)
        plot_data[method] = method_data
    return plot_data

for nameOfEach, each in dfs.items():

    e1_plot_data = prepare_plot_data(each, alldf, 'e1', methods, sizes)
    e3_plot_data = prepare_plot_data(each, alldf, 'e3', methods, sizes)

    # 设置绘图风格
    plt.style.use('default')
    fig, axes = plt.subplots(2, 1, figsize=(10, 24), sharex=True)

    # 设置颜色
    colors = plt.cm.Set3(np.linspace(0, 1, len(methods)))

    # 绘制 e1
    x = np.arange(len(sizes))
    width = 0.13

    # 设置 y 轴为对数尺度
    axes[0].set_xscale('log')
    axes[1].set_xscale('log')

    # 绘制 e1 并标注数值
    for i, method in enumerate(methods):
        bars = axes[0].barh(x + i * width, e1_plot_data[method], width, label=method, color=colors[i])
        # 在每个柱子上标注数值
        for j, (bar, value) in enumerate(zip(bars, e1_plot_data[method])):
            if not np.isnan(value):
                axes[0].text(bar.get_width() * 1.05, bar.get_y() + bar.get_height()/2,
                            f'{value:.0f}', ha='left', va='center', fontsize=6, rotation=0)

    axes[0].set_ylabel('Data Files', fontsize=10)
    axes[0].set_xlabel('Execution Time (μs) - Log Scale', fontsize=10)
    axes[0].set_title(f'e1 Experiment', fontsize=12)
    axes[0].set_yticks(x + width * (len(methods) - 1) / 2)
    axes[0].set_yticklabels([f'{s}' for s in sizes], fontsize=8, rotation=35)
    axes[0].grid(True, linestyle='--', alpha=0.6, which='both')

    # 绘制 e3 并标注数值
    for i, method in enumerate(methods):
        bars = axes[1].barh(x + i * width, e3_plot_data[method], width, label=method, color=colors[i])
        # 在每个柱子上标注数值
        for j, (bar, value) in enumerate(zip(bars, e3_plot_data[method])):
            if not np.isnan(value):
                axes[1].text(bar.get_width() * 1.05, bar.get_y() + bar.get_height()/2, 
                            f'{value:.0f}', ha='left', va='center', fontsize=6, rotation=0)

    axes[1].set_ylabel('Data Size', fontsize=10)
    axes[1].set_xlabel('Execution Time (μs) - Log Scale', fontsize=10)
    axes[1].set_title('e3 Experiment', fontsize=12)
    axes[1].set_yticks(x + width * (len(methods) - 1) / 2)
    axes[1].set_yticklabels([f'{s}' for s in sizes], fontsize=8, rotation=35)
    axes[1].grid(True, linestyle='--', alpha=0.6, which='both')

    # 添加总标题
    fig.suptitle(f'{file_names_dict[nameOfEach]} WASM File: Performance Comparison by File and Size (Log Scale)', 
                fontsize=16, y=0.98)
    
    # 添加图例（只在第一个subplot显示）
    handles, labels = axes[0].get_legend_handles_labels()
    fig.legend(handles, labels, loc='upper center', bbox_to_anchor=(0.5, 0.95), 
              ncol=min(3, len(methods)), fontsize=9)
    
    # 调整布局
    plt.tight_layout(rect=[0, 0, 1, 0.95])  # 为总标题留出空间
    
    # 保存图片
    plt.savefig(f'./host/result/e1_e3_e5_{nameOfEach}.png', dpi=300, bbox_inches='tight')

    # plt.show()
    
def prepare_plot_data_for_experiment(experiment, methods, sizes, dfs):
    plot_data = {}
    for file_name, df in dfs.items():
        file_data = {}
        for method in methods:
            method_data = []
            for size in sizes:
                # 获取对应实验、规模和方法的数据
                if method in ["io", "native opt"]:
                    values = alldf[(alldf['name'] == method) & (alldf['size'] == size) & (alldf['experiment'] == experiment)]['no_warmup_avg']
                else:
                    values = df[(df['name'] == method) & (df['size'] == size) & (df['experiment'] == experiment)]['no_warmup_avg']
                    if len(values) > 1: raise ValueError
                if len(values) > 0:
                    if method == 'native opt':
                        method_data.append(values.max())
                    else:
                        method_data.append(values.min())
                else:
                    method_data.append(np.nan)
            file_data[method] = method_data
        plot_data[file_name] = file_data
    return plot_data

# 为每个实验准备数据
e1_data = prepare_plot_data_for_experiment('e1', methods, sizes, dfs)
e3_data = prepare_plot_data_for_experiment('e3', methods, sizes, dfs)

# 设置绘图风格
plt.style.use('default')

# 为每个实验创建单独的图
for experiment, data_dict in [('e1', e1_data), ('e3', e3_data)]:
    # 创建图形，每个size一个subplot
    fig, axes = plt.subplots(len(sizes), 1, figsize=(10, 24), sharex=True)
    
    # 如果只有一个size，确保axes是数组
    if len(sizes) == 1:
        axes = [axes]
    
    # 设置颜色 - 为每个文件设置不同颜色
    colors = plt.cm.Set3(np.linspace(0, 1, len(dfs)))
    
    # 为每个size创建subplot
    for size_idx, size in enumerate(sizes):
        ax = axes[size_idx]
        ax.set_xscale('log')
        
        # 过滤掉io和native opt，只保留wasm相关方法
        wasm_methods = [method for method in methods if method not in ["io", "native opt"]]
        x = np.arange(len(wasm_methods) + 2)  # +2 为io和native opt预留位置
        
        width = 0.9 / len(dfs)  # 动态调整宽度，基于文件数量
        
        # 为每个文件绘制柱状图
        for file_idx, file_name in enumerate(dfs.keys()):
            file_values = []
            
            # 先添加io数据（只在第一个文件时添加）
            if file_idx == 1:
                io_value = data_dict[file_name]["io"][size_idx]
                file_values.append(io_value)
            else:
                file_values.append(np.nan)  # 其他文件在io位置留空
            
            # 添加native opt数据（只在第一个文件时添加）
            if file_idx == 1:
                native_opt_value = data_dict[file_name]["native opt"][size_idx]
                file_values.append(native_opt_value)
            else:
                file_values.append(np.nan)  # 其他文件在native opt位置留空
            
            # 添加wasm方法数据
            for method in wasm_methods:
                value = data_dict[file_name][method][size_idx]
                file_values.append(value)
            
            # 绘制柱状图
            bars = ax.barh(x + file_idx * width, file_values, width, 
                         label=file_name if size_idx == 0 else "",  # 只在第一个subplot显示图例
                         color=colors[file_idx])
            
            # 标注数值
            for bar_idx, (bar, value) in enumerate(zip(bars, file_values)):
                if not np.isnan(value):
                    ax.text(bar.get_width() * 1.005, bar.get_y() + bar.get_height()/2, 
                           f'{value:.0f}', ha='left', va='center', fontsize=6, rotation=0)
        
        # 准备横坐标标签（只取'('前的部分）
        x_labels = ["io", "native opt"]
        for method in wasm_methods:
            # 只取方法名中'('前的部分
            short_name = method.split('(')[0].strip()
            x_labels.append(short_name)
        
        ax.set_ylabel('Methods', fontsize=10)
        ax.set_xlabel('Execution Time (μs)', fontsize=10)
        ax.set_title(f'Size: {size}', fontsize=12)
        ax.set_yticks(x + width * (len(dfs) - 1) / 2)
        ax.set_yticklabels(x_labels, fontsize=8, rotation=35)
        ax.grid(True, linestyle='--', alpha=0.6, which='both')
    
    # 添加总标题
    fig.suptitle(f'{experiment.upper()} Experiment: Performance Comparison by Method and Size', 
                fontsize=16, y=0.98)
    
    # 添加图例（只在第一个subplot显示）
    handles, labels = axes[0].get_legend_handles_labels()
    fig.legend(handles, labels, loc='upper center', bbox_to_anchor=(0.5, 0.95), 
              ncol=min(3, len(dfs)), fontsize=9)
    
    # 调整布局
    plt.tight_layout(rect=[0, 0, 1, 0.95])  # 为总标题留出空间
    
    # 保存图片
    plt.savefig(f'./host/result/{experiment}_by_method_and_size.png', dpi=300, bbox_inches='tight')
    plt.close()


df1 = pd.read_csv("./host/result/e2_e4_e5/bidComponent100_usedonly_opt.csv")
df2 = pd.read_csv("./host/result/e2_e4_e5/bidComponent100_usedonly.csv")
df3 = pd.read_csv("./host/result/e2_e4_e5/bidComponent100.csv ")
df4 = pd.read_csv("./host/result/e2_e4_e5/bidComponent10000_usedonly_opt.csv ")
df5 = pd.read_csv("./host/result/e2_e4_e5/bidComponent10000_usedonly.csv ")
df6 = pd.read_csv("./host/result/e2_e4_e5/bidComponent10000.csv")

dfs = {'bidComponent100_usedonly_opt': df1, 'bidComponent100_usedonly': df2, 'bidComponent100': df3, 'bidComponent10000_usedonly_opt': df4, 'bidComponent10000_usedonly': df5, 'bidComponent10000': df6}
file_names_dict = {'bidComponent100_usedonly_opt': 'usedonly_func_size_opt_switch_every_100', 'bidComponent100_usedonly': 'usedonly_func_switch_every_100', 'bidComponent100': 'all_func_switch_every_100', 'bidComponent10000_usedonly_opt': 'usedonly_func_size_opt_switch_every_10000', 'bidComponent10000_usedonly': 'usedonly_func_switch_every_10000', 'bidComponent10000': 'all_func_switch_every_10000'}
alldf = pd.concat([df1, df2, df3, df4, df5, df6], ignore_index=True)


methods = [
    "io",
    "wasm opt2 dynamic reload",
    "wasm opt3 dynamic reload",
    "wasm opt4 dynamic reload"
]

# 定义数据规模
sizes = [1, 10, 100, 1000, 10000]
    
def prepare_plot_data_for_experiment(experiment, methods, sizes, dfs):
    plot_data = {}
    for file_name, df in dfs.items():
        file_data = {}
        for method in methods:
            method_data = []
            for size in sizes:
                # 获取对应实验、规模和方法的数据
                if method in ["io"]:
                    values = alldf[(alldf['name'] == method) & (alldf['size'] == size) & (alldf['experiment'] == experiment)]['no_warmup_avg']
                else:
                    values = df[(df['name'] == method) & (df['size'] == size) & (df['experiment'] == experiment)]['no_warmup_avg']
                    if len(values) > 1: raise ValueError
                if len(values) > 0:
                    method_data.append(values.min())
                else:
                    method_data.append(np.nan)
            file_data[method] = method_data
        plot_data[file_name] = file_data
    return plot_data

# 为每个实验准备数据
e1_data = prepare_plot_data_for_experiment('e2', methods, sizes, dfs)
e3_data = prepare_plot_data_for_experiment('e4', methods, sizes, dfs)

# 设置绘图风格
plt.style.use('default')

# 为每个实验创建单独的图
# for experiment, data_dict in [('e2', e1_data), ('e4', e3_data)]:
experiment = 'e4'
data_dict = e3_data
# 创建图形，每个size一个subplot
fig, axes = plt.subplots(len(sizes), 1, figsize=(10, 24), sharex=True, height_ratios=[0.25,0.25,0.25,0.125,0.125])

# 如果只有一个size，确保axes是数组
if len(sizes) == 1:
    axes = [axes]

# 设置颜色 - 为每个文件设置不同颜色
colors = plt.cm.Set3(np.linspace(0, 1, len(dfs)))

# 为每个size创建subplot
for size_idx, size in enumerate(sizes):
    ax = axes[size_idx]
    ax.set_xscale('log')
    
    # 过滤掉io和native opt，只保留wasm相关方法
    wasm_methods = [method for method in methods if method not in ["io"]]
    x = np.arange(len(wasm_methods) + 1)  # +2 为io和native opt预留位置
    
    if size in [1000,10000]:
        width = 0.9 / 3
    else:
        width = 0.9 / len(dfs)  # 动态调整宽度，基于文件数量
    
    # 为每个文件绘制柱状图
    for file_idx, file_name in enumerate(dfs.keys()):
        file_values = []
        
        # 先添加io数据（只在第一个文件时添加）
        if file_idx == 2:
            io_value = data_dict[file_name]["io"][size_idx]
            file_values.append(io_value)
        else:
            file_values.append(np.nan)  # 其他文件在io位置留空
        
        # 添加wasm方法数据
        for method in wasm_methods:
            value = data_dict[file_name][method][size_idx]
            file_values.append(value)
        
        # 绘制柱状图
        if file_idx in [3,4,5] and size in [1000,10000]:
            pass
        else:
            bars = ax.barh(x + file_idx * width, file_values, width, 
                        label=file_name if size_idx == 0 else "",  # 只在第一个subplot显示图例
                        color=colors[file_idx])
        
        # 标注数值
        for bar_idx, (bar, value) in enumerate(zip(bars, file_values)):
            if file_idx in [3,4,5] and size in [1000,10000]:
                pass
            else:
                if not np.isnan(value):
                    ax.text(bar.get_width() * 1.005, bar.get_y() + bar.get_height()/2, 
                            f'{value:.0f}', ha='left', va='center', fontsize=6, rotation=0)
    
    # 准备横坐标标签（只取'('前的部分）
    x_labels = ["io"]
    for method in wasm_methods:
        # 只取方法名中'('前的部分
        short_name = method.split(' dynamic reload')[0].strip()
        x_labels.append(short_name)
    
    ax.set_ylabel('Methods', fontsize=10)
    ax.set_xlabel('Execution Time (μs)', fontsize=10)
    ax.set_title(f'Switch Times: {size}', fontsize=12)
    ax.set_yticks(x + width * (len(dfs) - 1) / 2)
    ax.set_yticklabels(x_labels, fontsize=8, rotation=35)
    ax.grid(True, linestyle='--', alpha=0.6, which='both')

# 添加总标题
fig.suptitle(f'{experiment.upper()} Experiment: Performance Comparison by Method and Switch Times', 
            fontsize=16, y=0.98)

# 添加图例（只在第一个subplot显示）
handles, labels = axes[0].get_legend_handles_labels()
fig.legend(handles, labels, loc='upper center', bbox_to_anchor=(0.5, 0.95), 
            ncol=min(3, len(dfs)), fontsize=9)

# 调整布局
plt.tight_layout(rect=[0, 0, 1, 0.95])  # 为总标题留出空间

# 保存图片
plt.savefig(f'./host/result/{experiment}_by_method_and_size.png', dpi=300, bbox_inches='tight')
plt.close()


df1 = pd.read_csv("./host/result/e2_e4_e5/bidComponent100_usedonly_opt.csv")
df2 = pd.read_csv("./host/result/e2_e4_e5/bidComponent100_usedonly.csv")
df3 = pd.read_csv("./host/result/e2_e4_e5/bidComponent100.csv ")
df4 = pd.read_csv("./host/result/e2_e4_e5/bidComponent10000_usedonly_opt.csv ")
df5 = pd.read_csv("./host/result/e2_e4_e5/bidComponent10000_usedonly.csv ")
df6 = pd.read_csv("./host/result/e2_e4_e5/bidComponent10000.csv")

dfs = {'bidComponent100_usedonly_opt': df1, 'bidComponent100_usedonly': df2, 'bidComponent100': df3, 'bidComponent10000_usedonly_opt': df4, 'bidComponent10000_usedonly': df5, 'bidComponent10000': df6}
file_names_dict = {'bidComponent100_usedonly_opt': 'usedonly_func_size_opt_switch_every_100', 'bidComponent100_usedonly': 'usedonly_func_switch_every_100', 'bidComponent100': 'all_func_switch_every_100', 'bidComponent10000_usedonly_opt': 'usedonly_func_size_opt_switch_every_10000', 'bidComponent10000_usedonly': 'usedonly_func_switch_every_10000', 'bidComponent10000': 'all_func_switch_every_10000'}
alldf = pd.concat([df1, df2, df3, df4, df5, df6], ignore_index=True)


methods = [
    "io",
    "wasm opt2 dynamic reload",
    "wasm opt3 dynamic reload",
    "wasm opt4 dynamic reload"
]

# 定义数据规模
sizes = [100, 10000]
    
def prepare_plot_data_for_experiment(experiment, methods, sizes, dfs):
    plot_data = {}
    for file_name, df in dfs.items():
        file_data = {}
        for method in methods:
            method_data = []
            for size in sizes:
                # 获取对应实验、规模和方法的数据
                if method in ["io"]:
                    values = alldf[(alldf['name'] == method) & (alldf['size'] == size) & (alldf['experiment'] == experiment)]['no_warmup_avg']
                else:
                    values = df[(df['name'] == method) & (df['size'] == size) & (df['experiment'] == experiment)]['no_warmup_avg']
                    if len(values) > 1: raise ValueError
                if len(values) > 0:
                    method_data.append(values.min())
                else:
                    method_data.append(np.nan)
            file_data[method] = method_data
        plot_data[file_name] = file_data
    return plot_data

# 为每个实验准备数据
e1_data = prepare_plot_data_for_experiment('e2', methods, sizes, dfs)
e3_data = prepare_plot_data_for_experiment('e4', methods, sizes, dfs)

# 设置绘图风格
plt.style.use('default')

# 为每个实验创建单独的图
# for experiment, data_dict in [('e2', e1_data), ('e4', e3_data)]:
experiment = 'e2'
data_dict = e1_data
# 创建图形，每个size一个subplot
fig, axes = plt.subplots(len(sizes), 1, figsize=(10, 12), sharex=True)

# 如果只有一个size，确保axes是数组
if len(sizes) == 1:
    axes = [axes]

# 设置颜色 - 为每个文件设置不同颜色
colors = plt.cm.Set3(np.linspace(0, 1, len(dfs)))

# 为每个size创建subplot
for size_idx, size in enumerate(sizes):
    ax = axes[size_idx]
    ax.set_xscale('log')
    
    # 过滤掉io和native opt，只保留wasm相关方法
    wasm_methods = [method for method in methods if method not in ["io"]]
    x = np.arange(len(wasm_methods) + 1)  # +2 为io和native opt预留位置
    
    width = 0.9 / 3 # 动态调整宽度，基于文件数量
    # max_value = math.inf
    # 为每个文件绘制柱状图
    for file_idx, file_name in enumerate(dfs.keys()):
        file_values = []
        
        # 先添加io数据（只在第一个文件时添加）
        if file_idx == 1:
            io_value = data_dict[file_name]["io"][size_idx]
            file_values.append(io_value)
        else:
            file_values.append(np.nan)  # 其他文件在io位置留空
        
        # 添加wasm方法数据
        for method in wasm_methods:
            value = data_dict[file_name][method][size_idx]
            file_values.append(value)
        
        # 绘制柱状图
        bars = ax.barh(x + file_idx%3 * width, file_values, width, 
                        label=file_name if size_idx == 0 else "",  # 只在第一个subplot显示图例
                        color=colors[file_idx])
        
        # 标注数值
        for bar_idx, (bar, value) in enumerate(zip(bars, file_values)):
            if not np.isnan(value):
                ax.text(bar.get_width() * 1.005, bar.get_y() + bar.get_height()/2, 
                        f'{value:.0f}', ha='left', va='center', fontsize=6, rotation=0)
                
        # print(file_values)
        # max_value = min(max([v for v in file_values if not np.isnan(v)]), max_value)
    
    # 准备横坐标标签（只取'('前的部分）
    x_labels = ["io"]
    for method in wasm_methods:
        # 只取方法名中'('前的部分
        short_name = method.split(' dynamic reload')[0].strip()
        x_labels.append(short_name)
    
    ax.set_ylabel('Methods', fontsize=10)
    ax.set_xlabel('Execution Time (μs)', fontsize=10)
    ax.set_title(f'Bids per Switch: {size}', fontsize=12)
    ax.set_yticks(x + width * (3 - 1) / 2)
    ax.set_yticklabels(x_labels, fontsize=8, rotation=35)
    ax.grid(True, linestyle='--', alpha=0.6, which='both')
    # ax.set_xlim(0, max_value * 1.1)

# 添加总标题
fig.suptitle(f'{experiment.upper()} Experiment: Performance Comparison by Method and Bids per Switch', 
            fontsize=16, y=0.98)

# 添加图例（只在第一个subplot显示）
handles, labels = axes[0].get_legend_handles_labels()
fig.legend(handles, labels, loc='upper center', bbox_to_anchor=(0.5, 0.95), 
            ncol=min(3, len(dfs)), fontsize=9)

# 调整布局
plt.tight_layout(rect=[0, 0, 1, 0.95])  # 为总标题留出空间

# 保存图片
plt.savefig(f'./host/result/{experiment}_by_method_and_size.png', dpi=300, bbox_inches='tight')
plt.close()

# 读取 CSV
df1 = pd.read_csv("./host/result/e2_e4_e5/bidComponent100_usedonly_opt.csv")
df2 = pd.read_csv("./host/result/e2_e4_e5/bidComponent100_usedonly.csv")
df3 = pd.read_csv("./host/result/e2_e4_e5/bidComponent100.csv")
df4 = pd.read_csv("./host/result/e2_e4_e5/bidComponent10000_usedonly_opt.csv")
df5 = pd.read_csv("./host/result/e2_e4_e5/bidComponent10000_usedonly.csv")
df6 = pd.read_csv("./host/result/e2_e4_e5/bidComponent10000.csv")

dfs = {'bidComponent100_usedonly': df2, 'bidComponent10000_usedonly': df5}
file_names_dict = {
    'bidComponent100_usedonly_opt': 'usedonly_func_size_opt_switch_every_100',
    'bidComponent100_usedonly': 'usedonly_func_switch_every_100',
    'bidComponent100': 'all_func_switch_every_100',
    'bidComponent10000_usedonly_opt': 'usedonly_func_size_opt_switch_every_10000',
    'bidComponent10000_usedonly': 'usedonly_func_switch_every_10000',
    'bidComponent10000': 'all_func_switch_every_10000'
}
alldf = pd.concat([df1, df2, df3, df4, df5, df6], ignore_index=True)

# 方法和数据规模
methods = [
    "io",
    "wasm opt2 dynamic reload",
    "wasm opt3 dynamic reload",
    "wasm opt4 dynamic reload"
]

def prepare_plot_data(onedata: pd.DataFrame, alldata: pd.DataFrame, experiment, methods, sizes):
    plot_data = {}
    for method in methods:
        method_data = []
        for size in sizes:
            if method in ["io", "native opt"]:
                values = alldata[(alldata['name'] == method) &
                                 (alldata['size'] == size) &
                                 (alldata['experiment'] == experiment)]['no_warmup_avg']
            else:
                values = onedata[(onedata['name'] == method) &
                                 (onedata['size'] == size) &
                                 (onedata['experiment'] == experiment)]['no_warmup_avg']
                if len(values) > 1: raise ValueError(f"Duplicate values for {method}, size {size}, experiment {experiment}")
            if len(values) > 0:
                method_data.append(values.max() if method == 'native opt' else values.min())
            else:
                method_data.append(np.nan)
        plot_data[method] = method_data
    return plot_data

for nameOfEach, each in dfs.items():
    if nameOfEach == 'bidComponent100_usedonly':
        sizes2 = [10000]
        sizes4 = [1, 10, 100, 1000, 10000]
    else:
        sizes2 = [100]
        sizes4 = [1, 10, 100]
    e1_plot_data = prepare_plot_data(each, alldf, 'e2', methods, sizes2)
    e3_plot_data = prepare_plot_data(each, alldf, 'e4', methods, sizes4)

    plt.style.use('default')
    fig, axes = plt.subplots(2, 1, figsize=(10, 10), height_ratios=[0.2,0.8], sharex=True)
    colors = plt.cm.Set3(np.linspace(0, 1, len(methods)))

    x2 = np.arange(len(sizes2))
    x4 = np.arange(len(sizes4))
    width = 0.20  # 柱子宽度

    # 绘制 e1
    axes[0].set_xscale('log')
    for i, method in enumerate(methods):
        bars = axes[0].barh(x2 + i * width, e1_plot_data[method], height=width, label=method, color=colors[i])
        for bar, value in zip(bars, e1_plot_data[method]):
            if not np.isnan(value):
                axes[0].text(bar.get_width() * 1.001, bar.get_y() + bar.get_height()/2,
                             f'{value:.0f}', ha='left', va='center', fontsize=6)
    axes[0].set_ylabel('Bids per Switch', fontsize=10)
    axes[0].set_xlabel('Execution Time (μs) - Log Scale', fontsize=10)
    axes[0].set_title('e2 Experiment', fontsize=12)
    axes[0].set_yticks(x2 + width * (len(methods)-1)/2)
    axes[0].set_yticklabels([f'{s}' for s in sizes2], fontsize=8, rotation=35)
    axes[0].grid(True, linestyle='--', alpha=0.5, which='both')

    # 绘制 e3
    axes[1].set_xscale('log')
    for i, method in enumerate(methods):
        bars = axes[1].barh(x4 + i * width, e3_plot_data[method], height=width, label=method, color=colors[i])
        for bar, value in zip(bars, e3_plot_data[method]):
            if not np.isnan(value):
                axes[1].text(bar.get_width() * 1.005, bar.get_y() + bar.get_height()/2,
                             f'{value:.0f}', ha='left', va='center', fontsize=6)
    axes[1].set_ylabel('Bids per Switch', fontsize=10)
    axes[1].set_xlabel('Execution Time (μs) - Log Scale', fontsize=10)
    axes[1].set_title('e4 Experiment', fontsize=12)
    axes[1].set_yticks(x4 + width * (len(methods)-1)/2)
    axes[1].set_yticklabels([f'{s}' for s in sizes4], fontsize=8, rotation=35)
    axes[1].grid(True, linestyle='--', alpha=0.5, which='both')

    # 总标题和图例
    fig.suptitle(f'{file_names_dict[nameOfEach]} WASM File:\nPerformance Comparison by File and Bids per Switch (Log Scale)', fontsize=16)
    handles, labels = axes[0].get_legend_handles_labels()
    fig.legend(handles, labels, loc='upper center', bbox_to_anchor=(0.5,0.93), ncol=min(3, len(methods)), fontsize=9)

    plt.tight_layout(rect=[0, 0, 1, 0.95])
    plt.savefig(f'./host/result/e2_e4_e6_{nameOfEach}.png', dpi=300, bbox_inches='tight')
    # plt.show()
