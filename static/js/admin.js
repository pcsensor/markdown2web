(function() {
  const panel = document.getElementById('build-progress-panel');
  if (!panel) return;

  const progressBar = document.getElementById('progress-bar');
  const progressText = document.getElementById('progress-text');
  const progressPercent = document.getElementById('progress-percent');
  const currentJobText = document.getElementById('progress-current-job');
  const statusChip = document.getElementById('progress-status');

  let pollTimer = null;
  let lastSessionId = null;
  let hasShownForThisSession = false;

  async function checkProgress() {
    try {
      const response = await fetch('/admin/build-progress');
      if (!response.ok) return;
      
      const data = await response.json();
      
      // 如果 session_id 改变了（说明有新任务批次），重置追踪状态
      if (data.session_id !== lastSessionId) {
        lastSessionId = data.session_id;
        hasShownForThisSession = false;
      }

      // 如果当前 session_id 为 0（初始状态或重置），则始终隐藏
      if (!data.session_id || data.session_id === 0) {
        panel.style.display = 'none';
        return;
      }

      // 显示条件：
      // 1. 任务正在运行
      // 2. 任务已完成，但我们是在本次会话中看到它完成的（给予 3s 反馈时间）
      const shouldShow = data.is_running || (hasShownForThisSession && data.completed_jobs >= data.total_jobs);

      if (shouldShow) {
        panel.style.display = 'block';
        if (data.is_running) hasShownForThisSession = true; // 只要见过运行，就标记
        
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
          // 已完成态
          currentJobText.textContent = '所有媒体任务已完成';
          statusChip.textContent = '已完成';
          statusChip.style.background = 'rgba(20, 184, 166, 0.1)';
          statusChip.style.color = 'var(--accent-2)';
          
          // 完成后 3 秒自动进入“已读不回”状态
          setTimeout(() => {
            if (!data.is_running && data.session_id === lastSessionId) {
              panel.style.display = 'none';
              // 注意：这里不要重置 hasShownForThisSession，以防刷新后再次显示
            }
          }, 3000);
        }

        if (data.last_error) {
          console.error('Build worker error:', data.last_error);
        }
      } else {
        // 如果任务早已完成且我们刚才刷新了页面，则静默
        panel.style.display = 'none';
      }
    } catch (err) {
      console.error('Failed to fetch build progress', err);
    }
  }

  // 每 1.5 秒轮询一次
  pollTimer = setInterval(checkProgress, 1500);
  checkProgress(); 

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
      const csrfToken = form.querySelector('input[name="_csrf"]')?.value || '';

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
      if (csrfToken) {
        xhr.setRequestHeader('X-CSRF-Token', csrfToken);
      }
      xhr.send(formData);
    });
  }

  wireUploadForm('upload-markdown-form', 'markdown-progress-container', 'markdown-progress-bar', 'markdown-progress-text');
  wireUploadForm('upload-asset-form', 'asset-progress-container', 'asset-progress-bar', 'asset-progress-text');
})();
