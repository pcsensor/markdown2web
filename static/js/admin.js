(function() {
  const panel = document.getElementById('build-progress-panel');
  if (!panel) return;

  const progressBar = document.getElementById('progress-bar');
  const progressText = document.getElementById('progress-text');
  const progressPercent = document.getElementById('progress-percent');
  const currentJobText = document.getElementById('progress-current-job');
  const statusChip = document.getElementById('progress-status');

  let pollTimer = null;

  async function checkProgress() {
    try {
      const response = await fetch('/admin/build-progress');
      if (!response.ok) return;
      
      const data = await response.json();
      
      if (data.is_running || data.completed_jobs < data.total_jobs) {
        panel.style.display = 'block';
        
        const total = data.total_jobs || 0;
        const completed = data.completed_jobs || 0;
        const percentage = total > 0 ? Math.round((completed / total) * 100) : 0;
        
        progressBar.style.width = percentage + '%';
        progressPercent.textContent = percentage + '%';
        progressText.textContent = `已完成 ${completed} / 共 ${total} 个任务`;
        
        if (data.current_job) {
          const filename = data.current_job.destination.split('/').pop();
          currentJobText.textContent = `正在处理: ${filename}`;
          statusChip.textContent = '执行中';
          statusChip.style.background = 'rgba(37, 99, 235, 0.1)';
          statusChip.style.color = 'var(--accent)';
        } else if (data.is_running) {
          currentJobText.textContent = '等待任务启动...';
        } else {
          currentJobText.textContent = '所有媒体任务已完成';
          statusChip.textContent = '已完成';
          statusChip.style.background = 'rgba(20, 184, 166, 0.1)';
          statusChip.style.color = 'var(--accent-2)';
          
          // 完成后 3 秒隐藏
          setTimeout(() => {
            if (!data.is_running) panel.style.display = 'none';
          }, 3000);
        }

        if (data.last_error) {
          console.error('Build worker error:', data.last_error);
        }
      } else {
        panel.style.display = 'none';
      }
    } catch (err) {
      console.error('Failed to fetch build progress', err);
    }
  }

  // 每 1.5 秒轮询一次
  pollTimer = setInterval(checkProgress, 1500);
  checkProgress(); // 立即执行一次
})();
