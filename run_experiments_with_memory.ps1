# run_experiments_with_memory.ps1

$experiments = @(
    # e1 & e3 实验组
    @{Name="e1_default"; Args="nexmark-data/bid e1 100 25 ./host/result/e1_e3_e5/experiment_results default"},
    @{Name="e3_default"; Args="nexmark-data/bid e3 100 25 ./host/result/e1_e3_e5/experiment_results default"},
    @{Name="e1_usedonly"; Args="nexmark-data/bid e1 100 25 ./host/result/e1_e3_e5/experiment_results_usedonly usedonly"},
    @{Name="e3_usedonly"; Args="nexmark-data/bid e3 100 25 ./host/result/e1_e3_e5/experiment_results_usedonly usedonly"},
    @{Name="e1_usedonly_opt"; Args="nexmark-data/bid e1 100 25 ./host/result/e1_e3_e5/experiment_results_usedonly_opt usedonly_opt"},
    @{Name="e3_usedonly_opt"; Args="nexmark-data/bid e3 100 25 ./host/result/e1_e3_e5/experiment_results_usedonly_opt usedonly_opt"},
    
    # e2 & e4 实验组 - bidComponent100
    @{Name="e2_bidComponent100"; Args="nexmark-data/bidComponent100 e2 100 25 ./host/result/e2_e4_e5/bidComponent100"},
    @{Name="e4_bidComponent100"; Args="nexmark-data/bidComponent100 e4 100 25 ./host/result/e2_e4_e5/bidComponent100"},
    
    # e2 & e4 实验组 - bidComponent10000
    @{Name="e2_bidComponent10000"; Args="nexmark-data/bidComponent10000 e2 100 25 ./host/result/e2_e4_e5/bidComponent10000"},
    @{Name="e4_bidComponent10000"; Args="nexmark-data/bidComponent10000 e4 100 25 ./host/result/e2_e4_e5/bidComponent10000"},
    
    # e2 & e4 实验组 - bidComponent100_usedonly
    @{Name="e2_bidComponent100_usedonly"; Args="nexmark-data/bidComponent100_usedonly e2 100 25 ./host/result/e2_e4_e5/bidComponent100_usedonly"},
    @{Name="e4_bidComponent100_usedonly"; Args="nexmark-data/bidComponent100_usedonly e4 100 25 ./host/result/e2_e4_e5/bidComponent100_usedonly"},
    
    # e2 & e4 实验组 - bidComponent10000_usedonly
    @{Name="e2_bidComponent10000_usedonly"; Args="nexmark-data/bidComponent10000_usedonly e2 100 25 ./host/result/e2_e4_e5/bidComponent10000_usedonly"},
    @{Name="e4_bidComponent10000_usedonly"; Args="nexmark-data/bidComponent10000_usedonly e4 100 25 ./host/result/e2_e4_e5/bidComponent10000_usedonly"},
    
    # e2 & e4 实验组 - bidComponent100_usedonly_opt
    @{Name="e2_bidComponent100_usedonly_opt"; Args="nexmark-data/bidComponent100_usedonly_opt e2 100 25 ./host/result/e2_e4_e5/bidComponent100_usedonly_opt"},
    @{Name="e4_bidComponent100_usedonly_opt"; Args="nexmark-data/bidComponent100_usedonly_opt e4 100 25 ./host/result/e2_e4_e5/bidComponent100_usedonly_opt"},
    
    # e2 & e4 实验组 - bidComponent10000_usedonly_opt
    @{Name="e2_bidComponent10000_usedonly_opt"; Args="nexmark-data/bidComponent10000_usedonly_opt e2 100 25 ./host/result/e2_e4_e5/bidComponent10000_usedonly_opt"},
    @{Name="e4_bidComponent10000_usedonly_opt"; Args="nexmark-data/bidComponent10000_usedonly_opt e4 100 25 ./host/result/e2_e4_e5/bidComponent10000_usedonly_opt"}
)

# 创建结果目录（如果不存在）
$resultDirs = @(
    ".\host\result\e1_e3_e5",
    ".\host\result\e2_e4_e5"
)
foreach ($dir in $resultDirs) {
    if (!(Test-Path $dir)) {
        New-Item -ItemType Directory -Path $dir -Force
    }
}

# 初始化汇总文件
$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$summaryFile = "memory_summary_$timestamp.csv"
"Experiment,StartTime,EndTime,Duration(seconds),PeakMemory(MB),AvgMemory(MB),ExitCode" | Out-File -FilePath $summaryFile

# 运行实验
$totalExperiments = $experiments.Count
$currentExperiment = 0

foreach ($exp in $experiments) {
    $currentExperiment++
    Write-Host "`n========================================" -ForegroundColor Cyan
    Write-Host "[$currentExperiment/$totalExperiments] Starting $($exp.Name)" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
    
    $startTime = Get-Date
    $peakMem = 0
    $memReadings = @()
    $avgMem = 0
    
    # 启动实验
    $process = Start-Process -FilePath "cargo" `
        -ArgumentList "r --release --manifest-path=host/Cargo.toml -- $($exp.Args)" `
        -PassThru -NoNewWindow
    
    # 监控内存
    while (!$process.HasExited) {
        try {
            $proc = Get-Process -Id $process.Id -ErrorAction Stop
            $mem = [math]::Round($proc.WorkingSet64 / 1MB, 2)
            $memReadings += $mem
            if ($mem -gt $peakMem) { $peakMem = $mem }
            
            $elapsed = (Get-Date) - $startTime
            Write-Host "`r$($exp.Name) - Elapsed: $($elapsed.ToString('hh\:mm\:ss')) - Current: $mem MB - Peak: $peakMem MB" -NoNewline
            Start-Sleep -Milliseconds 500
        } catch {
            break
        }
    }
    
    $endTime = Get-Date
    $duration = ($endTime - $startTime).TotalSeconds
    
    # 计算平均内存
    if ($memReadings.Count -gt 0) {
        $avgMem = [math]::Round(($memReadings | Measure-Object -Average).Average, 2)
    }
    
    # 等待进程完全结束
    $process.WaitForExit()
    $exitCode = $process.ExitCode
    
    # 输出单个实验结果
    Write-Host "`n"  # 换行
    Write-Host "$($exp.Name) Complete!" -ForegroundColor Green
    Write-Host "  Duration: $([math]::Round($duration, 2)) seconds"
    Write-Host "  Peak Memory: $peakMem MB"
    Write-Host "  Average Memory: $avgMem MB"
    Write-Host "  Exit Code: $exitCode"
    
    # 写入汇总文件
    "$($exp.Name),$startTime,$endTime,$([math]::Round($duration, 2)),$peakMem,$avgMem,$exitCode" | Out-File -FilePath $summaryFile -Append
    
    # 实验间暂停，让系统稳定
    Write-Host "`nCooling down for 3 seconds..." -ForegroundColor DarkGray
    Start-Sleep -Seconds 3
}

# 最终汇总
Write-Host "`n========================================" -ForegroundColor Magenta
Write-Host "All experiments completed!" -ForegroundColor Magenta
Write-Host "========================================" -ForegroundColor Magenta
Write-Host "Summary saved to: $summaryFile" -ForegroundColor Yellow

# 显示汇总表
Write-Host "`nMemory Usage Summary:" -ForegroundColor Cyan
Write-Host "--------------------" -ForegroundColor Cyan
Import-Csv $summaryFile | Format-Table Experiment, Duration, PeakMemory, AvgMemory -AutoSize