import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

import matplotlib.pyplot as plt
import pandas as pd
import numpy as np

root_path = './host/result/e1_e3_e5/'
to_drop_columns = ['timestamp','amount','warmup','duration','amount_avg']
file_names = [('experiment_results', 'all_func'), ('experiment_results_usedonly', 'usedonly_func'), ('experiment_results_usedonly_opt', 'usedonly_func_size_opt')]

dfs = pd.DataFrame()
for (file_name, label_name) in file_names:
    df = pd.read_csv(f"{root_path}{file_name}.csv").drop(columns=to_drop_columns)
    df['file_name'] = file_name
    dfs = pd.concat([dfs, df], ignore_index=True)

same_names = ['io', 'native opt']
names = [name for name in dfs['name'].unique() if name not in same_names]

sizes = dfs['size'].unique()
experiments = dfs['experiment'].unique()
fig, axes = plt.subplots(1, len(sizes), figsize=(20, 15), squeeze=False, sharey=True)
axes = axes.flatten()

for i, (ax, size) in enumerate(zip(axes, sizes)):
    bar_positions = np.arange(len(sizes))
    bar_width = 0.15

    size_mask = dfs['size'] == size
    
    for (file_name, label_name) in file_names:
        file_name_mask = dfs['file_name'] == file_name
        
        name_data = {}
        for name in same_names:
            name_mask = dfs['name'] == name
            name_values = []
            
            combined_mask = name_mask & size_mask & file_name_mask
            print(dfs.loc[combined_mask, 'no_warmup_avg'])
            # exit()
            name_values.append(dfs.loc[combined_mask, 'no_warmup_avg'])
            
        name_data[name] = name_values
    io = name_data['io'] * (len(names)+1)
    native_opt = name_data['native opt']
    print(io, native_opt)
    
    native_opt_bar = ax.bar(bar_positions + -1 * bar_width, native_opt, width=bar_width, label='native opt', alpha=0.8, edgecolor='black', linewidth=1)
    for idx, rect in enumerate(native_opt_bar):
        height = rect.get_height()  # 总高度（包括底部）
        ax.text(rect.get_x() + rect.get_width()/2., height,
                f'{native_opt[idx]:.1f}',
                ha='center', va='bottom', fontsize=5, rotation=0)
    
    for (file_name, label_name) in file_names:
        file_name_mask = dfs['file_name'] == file_name
        experiment_mask = dfs['experiment'] == experiment
        name_data = {}
        for name in names:
            name_mask = dfs['name'] == name
            name_values = []
            for size in sizes:
                size_mask = dfs['size'] == size
                combined_mask = name_mask & size_mask & experiment_mask & file_name_mask
                name_values.append(dfs.loc[combined_mask, 'no_warmup_avg'].sum())
            name_data[name] = name_values
        
        for i, name in enumerate(names):
            data = name_data[name]
            bars = ax.bar(bar_positions + i * bar_width, data, width=bar_width, label=label_name+": "+name.split(' (')[0], alpha=0.8, edgecolor='black', linewidth=1)
            
            # 为每个条形添加数值标签
            for idx, rect in enumerate(bars):
                height = rect.get_height()  # 总高度（包括底部）
                ax.text(rect.get_x() + rect.get_width()/2., height,
                        f'{data[idx]:.1f}',
                        ha='center', va='bottom', fontsize=5, rotation=0)
    
    io_pos = []
    for i in range(-1, len(names)):
        io_pos.extend(bar_positions + i * bar_width)
    print(io, len(io))
    print(io_pos, len(io_pos))
    io_bar = ax.bar(io_pos, io, width=bar_width, label='io', alpha=0.8, edgecolor='black', linewidth=1)
    for idx, rect in enumerate(io_bar):
        height = rect.get_height()  # 总高度（包括底部）
        ax.text(rect.get_x() + rect.get_width()/2., height,
                f'{io[idx]:.1f}',
                ha='center', va='bottom', fontsize=5, rotation=0)
    
    
    ax.set_ylim(10**(np.log10(min(io))- 0.1))
    ax.set_title(f'{experiment} Experiment: Performance Comparison (Log Scale)', fontsize=14)
    ax.set_yscale('log')
    ax.set_xlabel('Data Size', fontsize=12)
    ax.set_ylabel('Average Execution Time (μs) - Log Scale', fontsize=12)
    ax.set_xticks(bar_positions)
    ax.set_xticklabels([f'{s}' for s in sizes], fontsize=10, rotation=45)
    ax.legend(loc='upper left', fontsize=9)
    ax.grid(True, linestyle='--', alpha=0.6, which='both', axis='y')

plt.tight_layout()

plt.savefig(f'{root_path}e1_e3_e5_performance_comparison_log_scale.png', dpi=500, bbox_inches='tight')
plt.show()