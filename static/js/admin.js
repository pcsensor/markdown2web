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
      
      // 只有当有任务在运行，或者刚完成（且还没被后端重置）时才显示
      if (data.total_jobs > 0) {
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

  // --- 上传进度处理逻辑 ---
  function wireUploadForm(formId, containerId, barId, textId) {
    const form = document.getElementById(formId);
    if (!form) return;

    const container = document.getElementById(containerId);
    const bar = document.getElementById(barId);
    const text = document.getElementById(textId);
    const submitBtn = form.querySelector('button[type="submit"]');

    form.addEventListener('submit', function(e) {
      e.preventDefault();
      
      const formData = new FormData(form);
      const xhr = new XMLHttpRequest();

      // 显示进度条，禁用按钮
      container.style.display = 'flex';
      bar.style.width = '0%';
      text.textContent = '0%';
      submitBtn.disabled = true;
      submitBtn.style.opacity = '0.5';

      xhr.upload.addEventListener('progress', function(e) {
        if (e.lengthComputable) {
          const percent = Math.round((e.loaded / e.total) * 100);
          bar.style.width = percent + '%';
          text.textContent = percent + '%';
        }
      });

      xhr.addEventListener('load', function() {
        if (xhr.status >= 200 && xhr.status < 300) {
          text.textContent = '完成!';
          // 上传成功后刷新页面以显示新内容
          setTimeout(() => window.location.reload(), 500);
        } else {
          alert('上传失败: ' + xhr.statusText);
          container.style.display = 'none';
          submitBtn.disabled = false;
          submitBtn.style.opacity = '1';
        }
      });

      xhr.addEventListener('error', function() {
        alert('网络错误，请重试');
        container.style.display = 'none';
        submitBtn.disabled = false;
        submitBtn.style.opacity = '1';
      });

      xhr.open('POST', form.action);
      xhr.send(formData);
    });
  }

  wireUploadForm('upload-markdown-form', 'markdown-progress-container', 'markdown-progress-bar', 'markdown-progress-text');
  wireUploadForm('upload-asset-form', 'asset-progress-container', 'asset-progress-bar', 'asset-progress-text');
})();
