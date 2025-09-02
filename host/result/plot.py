import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

root_path = ['./host/result/e1_e3_e5/', './host/result/e2_e4_e5/']
to_drop_columns = ['timestamp','amount','warmup','duration','amount_avg']
file_names = [[('experiment_results', 'all_func'), ('experiment_results_usedonly', 'usedonly_func'), ('experiment_results_usedonly_opt', 'usedonly_func_size_opt')], [('bidComponent100_usedonly_opt', 'usedonly_func_size_opt'), ('bidComponent100_usedonly', 'usedonly_func'), ('bidComponent100', 'all_func'), ('bidComponent10000_usedonly_opt', 'usedonly_func_size_opt'), ('bidComponent10000_usedonly', 'usedonly_func'), ('bidComponent10000', 'all_func')]]
same_names = ['io', 'native opt']

dfs = pd.DataFrame()
for idx, path in enumerate(root_path):
    for (file_name, label_name) in file_names[idx]:
        df = pd.read_csv(f"{root_path[idx]}{file_name}.csv").drop(columns=to_drop_columns)
        df['file_name'] = file_name
        df['label_name'] = label_name
        dfs = pd.concat([dfs, df], ignore_index=True)

names = [name for name in dfs['name'].unique() if name not in same_names]
sizes = dfs['size'].unique()
experiments = dfs['experiment'].unique()

for experiment in experiments:
    experiment_mask = dfs['experiment'] == experiment
    fig, axes = plt.subplots(1, len(sizes), figsize=(18, 8), squeeze=False)
    axes = axes.flatten()
    
    for ax, size in zip(axes, sizes):
        bar_positions = np.arange(len(names))
        bar_width = 0.15
        
        size_mask = dfs['size'] == size
        
        name_values = []
        for idx, name in enumerate(same_names):
            name_mask = dfs['name'] == name
            combined_mask = experiment_mask & name_mask & size_mask
            print(dfs.loc[combined_mask, 'no_warmup_avg'])
            print(dfs.loc[combined_mask, 'no_warmup_avg'].mean())
            name_values.append(dfs.loc[combined_mask, 'no_warmup_avg'].mean())
        print(name_values)
        
        native_opt_bar = ax.bar(bar_positions + 0 * bar_width, name_values[1], width=bar_width, label='native opt', alpha=0.8, edgecolor='black', linewidth=1)
        
        # ax.set_ylim(10**(np.log10(min(dfs['no_warmup_avg']))- 0.1))
        # ax.set_yscale('log')
        # ax.set_xlabel('Data Size', fontsize=12)
        # ax.set_ylabel('Average Execution Time (μs) - Log Scale', fontsize=12)
        # ax.set_xticks(bar_positions)
        # ax.set_xticklabels([f'{name.split(' (')[0]}' for name in names], fontsize=10, rotation=45)
        # ax.legend(loc='upper left', fontsize=9)
        # ax.grid(True, linestyle='--', alpha=0.6, which='both', axis='y')

    plt.tight_layout()
    plt.title(f'{experiment} & e5 Experiment: Performance Comparison (Log Scale)', fontsize=14)

    plt.savefig(f'./host/result/e1_e3_e5_performance_comparison_log_scale.png', dpi=500, bbox_inches='tight')
    plt.show()