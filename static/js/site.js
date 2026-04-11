const reduceMotionQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
const finePointerQuery = window.matchMedia('(pointer: fine)');
const prefersReducedMotion = () => reduceMotionQuery.matches;
const hasFinePointer = () => finePointerQuery.matches;

function markCurrentNav() {
  const current = window.location.pathname;
  document.querySelectorAll('.nav-link').forEach((link) => {
    const href = link.getAttribute('href');
    if (!href) return;
    const isCurrent = href === '/' ? current === '/' : current === href || current.startsWith(`${href}/`);
    link.classList.toggle('is-current', isCurrent);
  });
}

function wireScrollState() {
  const header = document.querySelector('.site-header');
  const progress = document.querySelector('.reading-progress');

  const update = () => {
    const y = window.scrollY;
    document.body.classList.toggle('has-scrolled', y > 10);
    if (header) header.classList.toggle('is-scrolled', y > 10);

    if (progress) {
      const doc = document.documentElement;
      const max = doc.scrollHeight - doc.clientHeight;
      const ratio = max > 0 ? Math.min(1, Math.max(0, y / max)) : 0;
      progress.style.setProperty('--progress', `${ratio}`);
    }
  };

  update();
  window.addEventListener('scroll', update, { passive: true });
}

function wireRevealAnimations() {
  const selectors = [
    '.hero-copy-block',
    '.hero-panel',
    '.section-head',
    '.card',
    '.list-card',
    '.note-hero',
    '.note-card',
    '.sidebar-card',
    '.stack-header',
    '.panel',
    '.auth-copy',
  ];
  const elements = [...new Set(selectors.flatMap((selector) => Array.from(document.querySelectorAll(selector))))];

  if (prefersReducedMotion()) {
    elements.forEach((element) => element.classList.add('is-visible'));
    return;
  }

  elements.forEach((element, index) => {
    element.classList.add('reveal-on-scroll');
    element.style.setProperty('--reveal-delay', `${Math.min(index * 35, 240)}ms`);
  });

  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          entry.target.classList.add('is-visible');
          observer.unobserve(entry.target);
        }
      });
    },
    { threshold: 0, rootMargin: '0px 0px -6% 0px' },
  );

  elements.forEach((element) => observer.observe(element));
}

function wireCursorBeacon() {
  const beacon = document.querySelector('.cursor-beacon');
  if (!beacon || prefersReducedMotion() || !hasFinePointer()) {
    document.body.classList.remove('cursor-beacon-enabled');
    return;
  }

  document.body.classList.add('cursor-beacon-enabled');

  const interactiveSelector = [
    '.interactive-card',
    'button',
    '.button',
    '.nav-link',
    '.toc-list a',
    '.link-list a',
    '.tag',
  ].join(', ');

  const state = {
    currentX: window.innerWidth / 2,
    currentY: window.innerHeight / 2,
    targetX: window.innerWidth / 2,
    targetY: window.innerHeight / 2,
    visible: false,
    active: false,
    rafId: null,
  };

  const animate = () => {
    state.rafId = window.requestAnimationFrame(animate);
    state.currentX += (state.targetX - state.currentX) * 0.18;
    state.currentY += (state.targetY - state.currentY) * 0.18;

    beacon.style.transform = `translate3d(${state.currentX}px, ${state.currentY}px, 0)`;
    beacon.classList.toggle('is-visible', state.visible);
    beacon.classList.toggle('is-active', state.active);
  };

  const setActiveFromTarget = (target) => {
    const interactive = target?.closest?.(interactiveSelector);
    state.active = Boolean(interactive);
  };

  animate();

  window.addEventListener(
    'pointermove',
    (event) => {
      state.targetX = event.clientX;
      state.targetY = event.clientY;
      state.visible = true;
      setActiveFromTarget(event.target);
    },
    { passive: true },
  );

  window.addEventListener('pointerdown', () => {
    beacon.classList.add('is-pressed');
  });

  window.addEventListener('pointerup', () => {
    beacon.classList.remove('is-pressed');
  });

  window.addEventListener('pointerleave', () => {
    state.visible = false;
    state.active = false;
  });

  window.addEventListener('blur', () => {
    state.visible = false;
    state.active = false;
  });
}

function clamp(value, min, max) {
  return Math.min(max, Math.max(min, value));
}

function wireMascot() {
  const mascot = document.querySelector('[data-mascot]');
  if (!mascot) return;

  const pupils = mascot.querySelectorAll('[data-pupil]');
  const mouth = mascot.querySelector('[data-mouth]');
  const label = mascot.querySelector('[data-mascot-label]');
  const maxPupilMove = 5;

  // 表情定义：嘴巴符号、瞳孔符号、CSS 类名、提示文字
  const expressions = [
    { mouth: '◡',  pupil: '●',  cls: '',              tip: '' },
    { mouth: '▽',  pupil: '★',  cls: 'expr-happy',    tip: '嘿嘿~' },
    { mouth: '○',  pupil: '◎',  cls: 'expr-surprised', tip: '哇！' },
    { mouth: 'ω',  pupil: '●',  cls: 'expr-smug',     tip: '略略略~' },
    { mouth: '◡',  pupil: '♥',  cls: 'expr-love',     tip: '喜欢你！' },
    { mouth: 'ε',  pupil: '●',  cls: 'expr-sleepy',   tip: 'zzZ...' },
  ];
  let exprIndex = 0;
  let labelTimer = 0;

  // 眼睛追踪鼠标
  const trackEyes = (event) => {
    const rect = mascot.getBoundingClientRect();
    const cx = rect.left + rect.width / 2;
    const cy = rect.top + rect.height * 0.38;
    const dx = event.clientX - cx;
    const dy = event.clientY - cy;
    const dist = Math.sqrt(dx * dx + dy * dy);
    if (dist < 1) return;

    const factor = Math.min(1, dist / 220);
    const px = (dx / dist) * maxPupilMove * factor;
    const py = (dy / dist) * maxPupilMove * factor;

    pupils.forEach((p) => {
      p.style.setProperty('--pupil-x', `${px.toFixed(1)}px`);
      p.style.setProperty('--pupil-y', `${py.toFixed(1)}px`);
    });
  };

  // 随机眨眼
  const blink = () => {
    if (mascot.classList.contains('expr-sleepy') || mascot.classList.contains('expr-happy')) return;
    mascot.classList.add('expr-blink');
    setTimeout(() => mascot.classList.remove('expr-blink'), 160);
  };
  const scheduleBlink = () => {
    const delay = 2400 + Math.random() * 4000;
    setTimeout(() => { blink(); scheduleBlink(); }, delay);
  };
  scheduleBlink();

  // 显示浮动标签
  const showLabel = (text) => {
    if (!label || !text) return;
    clearTimeout(labelTimer);
    label.textContent = text;
    label.classList.add('is-show');
    labelTimer = setTimeout(() => label.classList.remove('is-show'), 1800);
  };

  // 切换表情
  const switchExpression = () => {
    const prev = expressions[exprIndex];
    exprIndex = (exprIndex + 1) % expressions.length;
    const next = expressions[exprIndex];

    if (prev.cls) mascot.classList.remove(prev.cls);
    if (next.cls) mascot.classList.add(next.cls);

    if (mouth) mouth.textContent = next.mouth;
    pupils.forEach((p) => { p.textContent = next.pupil; });

    // 弹跳动画
    mascot.classList.remove('is-bouncing');
    requestAnimationFrame(() => mascot.classList.add('is-bouncing'));
    setTimeout(() => mascot.classList.remove('is-bouncing'), 460);

    showLabel(next.tip);
  };

  // 绑定事件
  if (!prefersReducedMotion() && hasFinePointer()) {
    window.addEventListener('pointermove', trackEyes, { passive: true });
  }
  mascot.addEventListener('click', switchExpression);
}

function wireButtons() {
  if (prefersReducedMotion()) return;
  document.querySelectorAll('button, .button, .nav-link').forEach((element) => {
    element.addEventListener('pointermove', (event) => {
      const rect = element.getBoundingClientRect();
      const px = ((event.clientX - rect.left) / rect.width) * 100;
      const py = ((event.clientY - rect.top) / rect.height) * 100;
      element.style.setProperty('--pointer-x', `${px}%`);
      element.style.setProperty('--pointer-y', `${py}%`);
    });
  });
}

function wireCardGlow() {
  if (prefersReducedMotion() || !hasFinePointer()) return;
  document.querySelectorAll('.card, .list-card').forEach((card) => {
    card.addEventListener('pointermove', (event) => {
      const rect = card.getBoundingClientRect();
      const x = event.clientX - rect.left;
      const y = event.clientY - rect.top;
      card.style.setProperty('--card-glow-x', `${x}px`);
      card.style.setProperty('--card-glow-y', `${y}px`);
    });
  });
}

function wireCodeBlocks() {
  document.querySelectorAll('.prose pre').forEach((block) => {
    if (block.querySelector('.copy-code-button')) return;
    const code = block.querySelector('code');
    if (!code) return;

    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'copy-code-button';
    button.textContent = 'Copy';
    button.setAttribute('aria-label', 'Copy code');
    button.setAttribute('data-skip-annotation', 'true');

    button.addEventListener('click', async () => {
      try {
        await navigator.clipboard.writeText(code.textContent ?? '');
        button.textContent = 'Copied';
        button.classList.add('copied');
        window.setTimeout(() => {
          button.textContent = 'Copy';
          button.classList.remove('copied');
        }, 1400);
      } catch (error) {
        console.warn('Copy failed', error);
      }
    });

    block.appendChild(button);
  });
}

function renderMath() {
  if (typeof window.katex === 'undefined') return;

  document.querySelectorAll('[data-math-style]').forEach((element) => {
    const source = element.textContent?.trim();
    if (!source || element.dataset.mathRendered === 'true') return;

    const displayMode = element.dataset.mathStyle === 'display';
    let target = element;

    if (element.tagName === 'CODE' && element.parentElement?.tagName === 'PRE') {
      const wrapper = document.createElement('div');
      wrapper.className = 'math-block';
      element.parentElement.replaceWith(wrapper);
      target = wrapper;
    }

    try {
      window.katex.render(source, target, {
        displayMode,
        throwOnError: false,
        strict: 'warn',
      });
      target.dataset.mathRendered = 'true';
      target.classList.add(displayMode ? 'math-display' : 'math-inline');
    } catch (error) {
      console.warn('KaTeX render failed', error);
    }
  });
}

function wireNoteAnnotations() {
  const article = document.querySelector('[data-note-article]');
  const root = article?.querySelector('[data-annotation-root]');
  const toolbar = document.querySelector('[data-annotation-toolbar]');
  const modal = document.querySelector('[data-annotation-modal]');
  const lane = document.querySelector('[data-annotation-comment-lane]');
  const emptyState = document.querySelector('[data-annotation-empty]');
  if (!article || !root || !toolbar || !lane || !modal) return;

  const noteSlug = article.dataset.noteSlug;
  const accountUrl = article.dataset.accountUrl || '/account';
  const viewerUsername = article.dataset.viewerUsername || '';
  const isAdmin = article.dataset.isAdmin === 'true';
  const enabled = article.dataset.annotationEnabled === 'true';
  const highlightButton = toolbar.querySelector('[data-annotation-highlight]');
  const commentButton = toolbar.querySelector('[data-annotation-comment]');
  const deleteCommentButton = toolbar.querySelector('[data-annotation-delete-comment]');
  const colorInput = toolbar.querySelector('[data-annotation-color]');
  const modalQuote = modal.querySelector('[data-annotation-modal-quote]');
  const modalInput = modal.querySelector('[data-annotation-comment-input]');
  const visibilitySelect = modal.querySelector('[data-annotation-visibility]');
  const modalSave = modal.querySelector('[data-annotation-comment-save]');
  const modalCloseButtons = modal.querySelectorAll('[data-annotation-modal-close]');
  if (!noteSlug || !highlightButton || !commentButton || !colorInput || !visibilitySelect || !modalInput || !modalQuote || !modalSave) return;

  const baseHtml = root.innerHTML;
  const stackedQuery = window.matchMedia('(max-width: 980px)');
  const state = {
    annotations: [],
    pending: null,
    activeAnnotationId: null,
    layoutFrame: 0,
    modalResolver: null,
  };

  const isOwnedAnnotation = (annotation) =>
    Boolean(annotation) && enabled && (isAdmin || annotation.username === viewerUsername);

  const targetElement = (target) => {
    if (target instanceof Element) return target;
    return target?.parentElement ?? null;
  };

  const closeCommentModal = (result = null) => {
    modal.hidden = true;
    document.body.classList.remove('annotation-modal-open');
    const resolver = state.modalResolver;
    state.modalResolver = null;
    if (resolver) resolver(result);
  };

  const openCommentModal = ({ quote, comment, visibility }) => {
    modal.hidden = false;
    document.body.classList.add('annotation-modal-open');
    modalQuote.textContent = `“${quote}”`;
    modalInput.value = comment ?? '';
    visibilitySelect.value = visibility === 'public' ? 'public' : 'private';
    requestAnimationFrame(() => {
      modalInput.focus();
      modalInput.setSelectionRange(modalInput.value.length, modalInput.value.length);
    });
    return new Promise((resolve) => {
      state.modalResolver = resolve;
    });
  };

  const hideToolbar = () => {
    toolbar.hidden = true;
    toolbar.classList.remove('is-visible');
    root.querySelectorAll('.note-annotation.is-active').forEach((element) => {
      element.classList.remove('is-active');
    });
    state.pending = null;
    state.activeAnnotationId = null;
  };

  const setToolbarPosition = (rect) => {
    toolbar.hidden = false;
    toolbar.style.left = '0px';
    toolbar.style.top = '0px';
    const toolbarRect = toolbar.getBoundingClientRect();
    const top = clamp(rect.top - toolbarRect.height - 12, 12, window.innerHeight - toolbarRect.height - 12);
    const left = clamp(
      rect.left + rect.width / 2 - toolbarRect.width / 2,
      12,
      window.innerWidth - toolbarRect.width - 12,
    );
    toolbar.style.left = `${left}px`;
    toolbar.style.top = `${top}px`;
    requestAnimationFrame(() => toolbar.classList.add('is-visible'));
  };

const updateToolbarState = () => {
    const annotation = state.pending?.annotation ?? null;
    const hasHighlight = Boolean(annotation?.color);
    const hasComment = Boolean(annotation?.comment);
    const canDeleteComment = annotation && hasComment && isOwnedAnnotation(annotation);
    highlightButton.textContent = hasHighlight ? '取消高亮' : '高亮';
    commentButton.textContent = hasComment ? '编辑评论' : '评论';
    colorInput.value = annotation?.color || colorInput.value || '#fde68a';
    if (deleteCommentButton) deleteCommentButton.hidden = !canDeleteComment;
  };

  const showToolbarForPending = (pending, rect) => {
    if (pending.annotation && !isOwnedAnnotation(pending.annotation)) return;
    state.pending = pending;
    state.activeAnnotationId = pending.annotation?.id ?? null;
    root.querySelectorAll('.note-annotation.is-active').forEach((element) => {
      element.classList.remove('is-active');
    });
    if (state.activeAnnotationId !== null) {
      annotationSegments(state.activeAnnotationId).forEach((element) => {
        element.classList.add('is-active');
      });
    }
    updateToolbarState();
    setToolbarPosition(rect);
  };

  const annotationSegments = (annotationId) =>
    Array.from(root.querySelectorAll(`[data-annotation-id="${annotationId}"]`));

  const annotationRect = (annotationId) => {
    const segments = annotationSegments(annotationId);
    if (!segments.length) return null;
    const rects = segments.map((segment) => segment.getBoundingClientRect());
    const left = Math.min(...rects.map((rect) => rect.left));
    const top = Math.min(...rects.map((rect) => rect.top));
    const right = Math.max(...rects.map((rect) => rect.right));
    const bottom = Math.max(...rects.map((rect) => rect.bottom));
    return {
      left,
      top,
      right,
      bottom,
      width: right - left,
      height: bottom - top,
    };
  };

  const textWalker = () =>
    document.createTreeWalker(
      root,
      NodeFilter.SHOW_TEXT,
      {
        acceptNode(node) {
          if (!node.nodeValue || !node.nodeValue.length) return NodeFilter.FILTER_REJECT;
          const parent = node.parentElement;
          if (!parent) return NodeFilter.FILTER_REJECT;
          if (parent.closest('.copy-code-button, script, style, textarea')) {
            return NodeFilter.FILTER_REJECT;
          }
          return NodeFilter.FILTER_ACCEPT;
        },
      },
    );

  const getOffsetsFromRange = (range) => {
    const startRange = range.cloneRange();
    startRange.selectNodeContents(root);
    startRange.setEnd(range.startContainer, range.startOffset);

    const endRange = range.cloneRange();
    endRange.selectNodeContents(root);
    endRange.setEnd(range.endContainer, range.endOffset);

    return {
      start: startRange.toString().length,
      end: endRange.toString().length,
    };
  };

  const collectSegments = (startOffset, endOffset) => {
    const walker = textWalker();
    const segments = [];
    let node;
    let cursor = 0;

    while ((node = walker.nextNode())) {
      const length = node.nodeValue.length;
      const nodeStart = cursor;
      const nodeEnd = cursor + length;
      const overlapStart = Math.max(startOffset, nodeStart);
      const overlapEnd = Math.min(endOffset, nodeEnd);

      if (overlapEnd > overlapStart) {
        segments.push({
          node,
          start: overlapStart - nodeStart,
          end: overlapEnd - nodeStart,
        });
      }
      cursor = nodeEnd;
    }

    return segments;
  };

  const normalizeText = (value) => value.replace(/\s+/g, ' ').trim();

  const segmentsText = (segments) =>
    segments
      .map(({ node, start, end }) => node.nodeValue.slice(start, end))
      .join('');

  const fullTextContent = () => {
    const walker = textWalker();
    let node;
    let text = '';

    while ((node = walker.nextNode())) {
      text += node.nodeValue;
    }

    return text;
  };

  const resolveAnnotationOffsets = (annotation) => {
    const directSegments = collectSegments(annotation.start_offset, annotation.end_offset);
    if (
      directSegments.length &&
      normalizeText(segmentsText(directSegments)) === normalizeText(annotation.quote)
    ) {
      return {
        start: annotation.start_offset,
        end: annotation.end_offset,
      };
    }

    const quote = annotation.quote.trim();
    if (!quote) return null;

    const haystack = fullTextContent();
    let cursor = 0;
    let bestIndex = -1;
    let bestDistance = Number.POSITIVE_INFINITY;

    while (cursor < haystack.length) {
      const index = haystack.indexOf(quote, cursor);
      if (index === -1) break;
      const distance = Math.abs(index - annotation.start_offset);
      if (distance < bestDistance) {
        bestDistance = distance;
        bestIndex = index;
      }
      cursor = index + Math.max(quote.length, 1);
    }

    if (bestIndex === -1) return null;
    return {
      start: bestIndex,
      end: bestIndex + quote.length,
    };
  };

  const wrapSegment = (segment, annotation, isFirst, isLast) => {
    const { node, start, end } = segment;
    const value = node.nodeValue;
    const before = value.slice(0, start);
    const middle = value.slice(start, end);
    const after = value.slice(end);
    const fragment = document.createDocumentFragment();

    if (before) fragment.appendChild(document.createTextNode(before));

    const span = document.createElement('span');
    span.className = 'note-annotation';
    span.dataset.annotationId = String(annotation.id);
    span.dataset.annotationStart = String(annotation.start_offset);
    span.dataset.annotationEnd = String(annotation.end_offset);
    if (annotation.color) {
      span.classList.add('has-highlight');
      span.style.setProperty('--annotation-color', annotation.color);
    }
    if (annotation.comment) {
      span.classList.add('has-comment');
      if (isFirst) span.classList.add('annotation-segment-start');
      if (isLast) span.classList.add('annotation-segment-end');
    }
    if (state.activeAnnotationId === annotation.id) {
      span.classList.add('is-active');
    }
    span.textContent = middle;
    fragment.appendChild(span);

    if (after) fragment.appendChild(document.createTextNode(after));
    node.parentNode.replaceChild(fragment, node);
  };

  const layoutComments = () => {
    if (state.layoutFrame) {
      cancelAnimationFrame(state.layoutFrame);
    }
    state.layoutFrame = requestAnimationFrame(() => {
      const comments = state.annotations.filter((annotation) => annotation.comment);
      lane.innerHTML = '';
      if (!comments.length) {
        lane.style.height = stackedQuery.matches ? 'auto' : '220px';
        if (emptyState) lane.appendChild(emptyState);
        return;
      }

      const articleRect = article.getBoundingClientRect();
      let lastBottom = 0;
      let isFirst = true;

      comments
        .map((annotation) => ({
          annotation,
          rect: annotationRect(annotation.id),
        }))
        .filter((item) => item.rect)
        .sort((left, right) => left.rect.top - right.rect.top)
        .forEach(({ annotation, rect }) => {
          const card = document.createElement('article');
          card.className = 'annotation-comment-card';
          card.dataset.annotationId = String(annotation.id);
          if (annotation.color) {
            card.style.setProperty('--annotation-color', annotation.color);
          }

          // 头部：头像字母 + 用户名 + 可见性徽章
          const head = document.createElement('div');
          head.className = 'annotation-comment-head';

          const avatar = document.createElement('span');
          avatar.className = 'annotation-comment-avatar';
          avatar.textContent = (annotation.username || '?')[0].toUpperCase();

          const meta = document.createElement('div');
          meta.className = 'annotation-comment-meta';

          const username = document.createElement('span');
          username.className = 'annotation-comment-username';
          username.textContent = annotation.username || '';

          const visibility = document.createElement('span');
          visibility.className = `annotation-comment-visibility ${annotation.visibility === 'public' ? 'is-public' : 'is-private'}`;
          visibility.textContent = annotation.visibility === 'public' ? '公开' : '私密';

          meta.append(username, visibility);
          head.append(avatar, meta);

          // 引用原文
          const quote = document.createElement('blockquote');
          quote.className = 'annotation-comment-quote';
          quote.textContent = annotation.quote;

          // 评论正文
          const body = document.createElement('p');
          body.className = 'annotation-comment-body';
          body.textContent = annotation.comment;

          card.append(head, quote, body);
          lane.appendChild(card);

          if (stackedQuery.matches) {
            card.style.top = 'auto';
            return;
          }

          const naturalTop = Math.max(0, rect.top - articleRect.top);
          const top = Math.max(isFirst ? Math.min(naturalTop, 8) : naturalTop, lastBottom);
          isFirst = false;
          card.style.top = `${top}px`;
          lastBottom = top + card.offsetHeight + 12;
        });

      lane.style.height = stackedQuery.matches
        ? 'auto'
        : `${Math.max(article.offsetHeight, lastBottom + 12, 220)}px`;
    });
  };

  const renderAnnotations = () => {
    root.innerHTML = baseHtml;

    state.annotations
      .slice()
      .sort((left, right) => {
        if (left.start_offset === right.start_offset) {
          return right.end_offset - left.end_offset;
        }
        return right.start_offset - left.start_offset;
      })
      .forEach((annotation) => {
        const offsets = resolveAnnotationOffsets(annotation);
        if (!offsets) return;
        const segments = collectSegments(offsets.start, offsets.end);
        if (!segments.length) return;
        segments
          .slice()
          .reverse()
          .forEach((segment, reverseIndex) => {
            const originalIndex = segments.length - 1 - reverseIndex;
            wrapSegment(
              segment,
              annotation,
              originalIndex === 0,
              originalIndex === segments.length - 1,
            );
          });
      });

    renderMath();
    wireCodeBlocks();
    wireAudioPlayers();
    wireVideoPlayers();
    layoutComments();
    window.dispatchEvent(new CustomEvent('note-content-rendered'));
  };

  const loadAnnotations = async () => {
    try {
      const response = await fetch(`/api/notes/${encodeURIComponent(noteSlug)}/annotations`, {
        headers: { Accept: 'application/json' },
      });
      if (!response.ok) {
        throw new Error(await response.text());
      }
      const data = await response.json();
      state.annotations = data.annotations ?? [];
      renderAnnotations();
    } catch (error) {
      console.warn('Failed to load annotations', error);
      renderAnnotations();
    }
  };

  const saveAnnotation = async (method, url, payload) => {
    const response = await fetch(url, {
      method,
      headers: {
        'Content-Type': 'application/json',
        Accept: 'application/json',
      },
      body: JSON.stringify(payload),
    });
    if (response.status === 401) {
      window.location.href = accountUrl;
      return null;
    }
    if (!response.ok) {
      throw new Error(await response.text());
    }
    if (response.status === 204) {
      return null;
    }
    return response.json();
  };

  const deleteAnnotation = async (annotationId) => {
    const response = await fetch(`/api/annotations/${annotationId}`, {
      method: 'DELETE',
      headers: { Accept: 'application/json' },
    });
    if (response.status === 401) {
      window.location.href = accountUrl;
      return;
    }
    if (!response.ok) {
      throw new Error(await response.text());
    }
  };

  const commit = async (work) => {
    try {
      await work();
      hideToolbar();
      window.getSelection()?.removeAllRanges();
      await loadAnnotations();
    } catch (error) {
      console.warn('Annotation action failed', error);
      window.alert('操作失败，请稍后重试。');
    }
  };

  const handleHighlightAction = () => {
    const pending = state.pending;
    if (!pending) return;

    commit(async () => {
      const annotation = pending.annotation;
      if (annotation?.color) {
        if (annotation.comment) {
          await saveAnnotation('PUT', `/api/annotations/${annotation.id}`, {
            color: null,
            comment: annotation.comment,
            visibility: annotation.visibility || 'private',
          });
        } else {
          await deleteAnnotation(annotation.id);
        }
        return;
      }

      const payload = {
        color: colorInput.value,
        comment: annotation?.comment ?? null,
        visibility: annotation?.comment ? (annotation.visibility || 'private') : 'private',
      };

      if (annotation) {
        await saveAnnotation('PUT', `/api/annotations/${annotation.id}`, payload);
      } else {
      await saveAnnotation('POST', `/api/notes/${encodeURIComponent(noteSlug)}/annotations`, {
        start_offset: pending.start,
        end_offset: pending.end,
        quote: pending.quote,
        ...payload,
        });
      }
    });
  };

  const handleCommentAction = () => {
    const pending = state.pending;
    if (!pending) return;

    openCommentModal({
      quote: pending.quote,
      comment: pending.annotation?.comment ?? '',
      visibility: pending.annotation?.visibility ?? 'private',
    }).then((result) => {
      if (!result) return;
      const trimmed = result.comment.trim();
      const annotation = pending.annotation;

      commit(async () => {
        if (annotation) {
          if (!trimmed) {
            if (annotation.color) {
              await saveAnnotation('PUT', `/api/annotations/${annotation.id}`, {
                color: annotation.color,
                comment: null,
                visibility: 'private',
              });
            } else {
              await deleteAnnotation(annotation.id);
            }
            return;
          }

          await saveAnnotation('PUT', `/api/annotations/${annotation.id}`, {
            color: annotation.color,
            comment: trimmed,
            visibility: result.visibility,
          });
          return;
        }

        if (!trimmed) return;

        await saveAnnotation('POST', `/api/notes/${encodeURIComponent(noteSlug)}/annotations`, {
          start_offset: pending.start,
          end_offset: pending.end,
          quote: pending.quote,
          color: null,
          comment: trimmed,
          visibility: result.visibility,
        });
      });
    });
  };

  const validSelectionRange = () => {
    const selection = window.getSelection();
    if (!selection || selection.rangeCount === 0 || selection.isCollapsed) return null;
    const range = selection.getRangeAt(0);
    const common = range.commonAncestorContainer.nodeType === Node.ELEMENT_NODE
      ? range.commonAncestorContainer
      : range.commonAncestorContainer.parentElement;
    if (!common || !root.contains(common)) return null;
    if (common.closest('pre, code, button, input, textarea')) return null;
    const quote = selection.toString().replace(/\s+/g, ' ').trim();
    if (!quote) return null;

    const { start, end } = getOffsetsFromRange(range);
    if (end <= start) return null;

    const ownedAnnotation =
      state.annotations.find(
        (item) =>
          item.username === viewerUsername &&
          item.start_offset === start &&
          item.end_offset === end,
      ) ?? null;

    return {
      start,
      end,
      quote,
      annotation: ownedAnnotation,
      rect: range.getBoundingClientRect(),
    };
  };

  const showSelectionToolbar = () => {
    if (!enabled) {
      hideToolbar();
      return;
    }
    const pending = validSelectionRange();
    if (!pending) {
      // 如果新的选中没有匹配到批注，清除之前的状态并隐藏工具栏
      hideToolbar();
      return;
    }
    showToolbarForPending(pending, pending.rect);
  };

  highlightButton.addEventListener('click', handleHighlightAction);
  commentButton.addEventListener('click', handleCommentAction);
  if (deleteCommentButton) {
    deleteCommentButton.addEventListener('click', () => {
      const pending = state.pending;
      if (!pending?.annotation?.comment) return;
      const annotation = pending.annotation;
      commit(async () => {
        if (annotation.color) {
          // 保留高亮，仅清除评论
          await saveAnnotation('PUT', `/api/annotations/${annotation.id}`, {
            color: annotation.color,
            comment: null,
            visibility: 'private',
          });
        } else {
          // 没有高亮，整条记录删除
          await deleteAnnotation(annotation.id);
        }
      });
    });
  }
  modalSave.addEventListener('click', () => {
    closeCommentModal({
      comment: modalInput.value,
      visibility: visibilitySelect.value,
    });
  });
  modalCloseButtons.forEach((button) => {
    button.addEventListener('click', () => closeCommentModal(null));
  });

  root.addEventListener('mouseup', (event) => {
    // 忽略右键释放，避免与 contextmenu 冲突
    if (event.button === 2) return;
    window.setTimeout(showSelectionToolbar, 0);
  });
  root.addEventListener('touchend', () => {
    window.setTimeout(showSelectionToolbar, 0);
  });

  const openOwnedAnnotationPanel = (annotationElement, forceNew = false) => {
    window.getSelection()?.removeAllRanges();
    const annotationId = Number(annotationElement.dataset.annotationId);
    const annotation = state.annotations.find((item) => item.id === annotationId);
    if (!annotation) return;
    const rect = annotationRect(annotationId) ?? annotationElement.getBoundingClientRect();

    if (isOwnedAnnotation(annotation) && !forceNew) {
      showToolbarForPending(
        {
          start: annotation.start_offset,
          end: annotation.end_offset,
          quote: annotation.quote,
          annotation,
        },
        rect,
      );
      return;
    }

    showToolbarForPending(
      {
        start: annotation.start_offset,
        end: annotation.end_offset,
        quote: annotation.quote,
        annotation: null,
      },
      rect,
    );
  };

  const annotationFromEvent = (event) => {
    const annotationElement = targetElement(event.target)?.closest('.note-annotation');
    if (!annotationElement) return null;
    if (!root.contains(annotationElement)) return null;
    return annotationElement;
  };

  const flashElements = (elements, cls) => {
    elements.forEach((el) => {
      el.classList.remove(cls);
      // 强制回流，确保移除后再加能重新触发动画
      void el.offsetWidth;
      el.classList.add(cls);
      el.addEventListener('animationend', () => el.classList.remove(cls), { once: true });
    });
  };

  root.addEventListener('click', (event) => {
    const annotationElement = annotationFromEvent(event);
    if (!annotationElement) return;
    if ('button' in event && event.button !== 0) return;
    event.preventDefault();

    const annotationId = Number(annotationElement.dataset.annotationId);
    const annotation = state.annotations.find((item) => item.id === annotationId);

    if (annotation?.comment) {
      const card = lane.querySelector(`[data-annotation-id="${annotationId}"]`);
      if (card) flashElements([card], 'is-flashing');
    }

    if (enabled) openOwnedAnnotationPanel(annotationElement, !isOwnedAnnotation(annotation));
  });

  document.addEventListener('mousedown', (event) => {
    if (event.button !== 2) return;
    const annotationElement = annotationFromEvent(event);
    if (!annotationElement || !enabled) return;
    event.preventDefault();
  }, true);

  document.addEventListener('contextmenu', (event) => {
    const annotationElement = annotationFromEvent(event);
    if (!annotationElement || !enabled) return;
    event.preventDefault();
    const annotationId = Number(annotationElement.dataset.annotationId);
    const annotation = state.annotations.find((item) => item.id === annotationId);
    openOwnedAnnotationPanel(annotationElement, !isOwnedAnnotation(annotation));
  }, true);

  // 单击评论卡片 → 闪烁文章内对应注释文字
  lane.addEventListener('click', (event) => {
    const card = targetElement(event.target)?.closest('[data-annotation-id]');
    if (!card) return;
    const annotationId = Number(card.dataset.annotationId);
    const segments = annotationSegments(annotationId);
    if (segments.length) flashElements(segments, 'is-flashing');
  });

  document.addEventListener('click', (event) => {
    if (toolbar.hidden) return;
    const target = targetElement(event.target);
    if (toolbar.contains(target) || root.contains(target)) return;
    hideToolbar();
  });

  window.addEventListener('resize', layoutComments, { passive: true });
  window.addEventListener('keydown', (event) => {
    if (event.key !== 'Escape') return;
    if (!modal.hidden) {
      closeCommentModal(null);
      return;
    }
    hideToolbar();
  });

  loadAnnotations();
}

function wireMobileNav() {
  const toggle = document.querySelector('[data-nav-toggle]');
  const menu = document.querySelector('[data-nav-menu]');
  const overlay = document.querySelector('[data-nav-overlay]');
  if (!toggle || !menu) return;

  const isOpen = () => toggle.getAttribute('aria-expanded') === 'true';

  const openMenu = () => {
    toggle.setAttribute('aria-expanded', 'true');
    menu.classList.add('is-open');
    if (overlay) overlay.classList.add('is-visible');
    document.body.style.overflow = 'hidden';
  };

  const closeMenu = () => {
    toggle.setAttribute('aria-expanded', 'false');
    menu.classList.remove('is-open');
    if (overlay) overlay.classList.remove('is-visible');
    document.body.style.overflow = '';
  };

  toggle.addEventListener('click', () => {
    if (isOpen()) closeMenu();
    else openMenu();
  });

  if (overlay) {
    overlay.addEventListener('click', closeMenu);
  }

  menu.querySelectorAll('.nav-link').forEach((link) => {
    link.addEventListener('click', closeMenu);
  });

  window.addEventListener('keydown', (event) => {
    if (event.key === 'Escape' && isOpen()) closeMenu();
  });
}


function wireTocScrollSpy() {
  const tocList = document.querySelector('.toc-sticky-panel .toc-list');
  const tocLinks = document.querySelectorAll('.toc-list a');
  if (!tocLinks.length || !tocList) return;

  let headings = [];

  const collectHeadings = () => {
    headings = [];
    tocLinks.forEach((link) => {
      const href = link.getAttribute('href');
      if (!href || !href.startsWith('#')) return;
      const id = href.slice(1);
      const anchor = document.getElementById(id);
      if (!anchor) return;
      const el = anchor.closest('h1, h2, h3, h4, h5, h6');
      if (!el) return;
      headings.push({ el, link });
    });
  };

  collectHeadings();
  if (!headings.length) return;

  let activeIndex = 0;

  const setActive = (index) => {
    if (index === activeIndex) return;
    activeIndex = index;
    headings.forEach((h, i) => {
      h.link.classList.toggle('is-active', i === index);
    });
    if (index >= 0) {
      const activeLink = headings[index].link;
      const linkCenter = activeLink.offsetTop - tocList.offsetTop + activeLink.offsetHeight / 2;
      tocList.scrollTop = linkCenter - tocList.clientHeight / 2;
    }
  };

  const TARGET = 100;

  const update = () => {
    if (!headings.length) return;
    let found = -1;
    for (let i = headings.length - 1; i >= 0; i--) {
      const rect = headings[i].el.getBoundingClientRect();
      if (rect.top <= TARGET) {
        found = i;
        break;
      }
    }
    setActive(found >= 0 ? found : 0);
  };

  update();
  window.addEventListener('scroll', update, { passive: true });

  window.addEventListener('note-content-rendered', () => {
    collectHeadings();
    if (headings.length) update();
  });
}
function init() {
  document.body.classList.add('js-ready');
  markCurrentNav();
  wireScrollState();
  wireTocScrollSpy();
  wireRevealAnimations();
  wireCursorBeacon();
  wireMascot();
  wireButtons();
  wireCardGlow();
  renderMath();
  wireNoteAnnotations();
  wireCodeBlocks();
  wireMobileNav();
  wireAccountToggle();
  wireAudioPlayers();
  wireVideoPlayers();
}

function wireAccountToggle() {
  const loginPanel = document.getElementById('login-panel');
  const registerPanel = document.getElementById('register-panel');
  const showRegister = document.getElementById('show-register');
  const showLogin = document.getElementById('show-login');
  if (!loginPanel || !registerPanel || !showRegister || !showLogin) return;

  const switchTo = (target) => {
    loginPanel.style.display = target === 'login' ? '' : 'none';
    registerPanel.style.display = target === 'register' ? '' : 'none';
    // Re-render Turnstile when panel becomes visible
    if (window.turnstile) {
      window.turnstile.reset();
    }
  };

  showRegister.addEventListener('click', (e) => {
    e.preventDefault();
    switchTo('register');
  });

  showLogin.addEventListener('click', (e) => {
    e.preventDefault();
    switchTo('login');
  });
}

function wireAudioPlayers() {
  document.querySelectorAll('[data-audio-player]').forEach((container) => {
    const audio = container.querySelector('[data-audio]');
    const playBtn = container.querySelector('[data-audio-play-btn]');
    if (!audio || !playBtn) return;

    const progressBar = container.querySelector('[data-audio-progress-bar]');
    const progressWrap = container.querySelector('[data-audio-progress-wrap]');
    const timeDisplay = container.querySelector('[data-audio-time]');
    const iconPlay = playBtn.querySelector('.audio-icon-play');
    const iconPause = playBtn.querySelector('.audio-icon-pause');

    const formatTime = (sec) => {
      if (isNaN(sec) || !isFinite(sec)) return '00:00';
      const m = Math.floor(sec / 60);
      const s = Math.floor(sec % 60);
      return `${String(m).padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
    };

    const setPlaybackUi = () => {
      const isPlaying = !audio.paused && !audio.ended;
      playBtn.classList.toggle('is-playing', isPlaying);
      playBtn.setAttribute('aria-label', isPlaying ? '暂停' : '播放');
    };

    const updateTimeDisplay = () => {
      const current = formatTime(audio.currentTime);
      const total = formatTime(audio.duration);
      if (timeDisplay) {
        timeDisplay.textContent = `${current}/${total}`;
      }
    };

    playBtn.addEventListener('click', () => {
      if (audio.paused) {
        if (audio.readyState === 0) {
          audio.load();
        }
        audio.play()
          .then(() => {
            setPlaybackUi();
          })
          .catch(() => {
            playBtn.disabled = true;
            playBtn.setAttribute('aria-label', '音频无法播放');
          });
      } else {
        audio.pause();
        setPlaybackUi();
      }
    });

    audio.addEventListener('play', () => {
      setPlaybackUi();
    });

    audio.addEventListener('pause', () => {
      setPlaybackUi();
    });

    audio.addEventListener('timeupdate', () => {
      const pct = (audio.currentTime / audio.duration) * 100;
      progressBar.style.width = `${pct}%`;
      updateTimeDisplay();
    });

    audio.addEventListener('loadedmetadata', () => {
      updateTimeDisplay();
    });

    audio.addEventListener('ended', () => {
      setPlaybackUi();
      progressBar.style.width = '0%';
      updateTimeDisplay();
    });

    audio.addEventListener('error', () => {
      playBtn.disabled = true;
      playBtn.setAttribute('aria-label', '音频无法播放');
      setPlaybackUi();
    });

    progressWrap.addEventListener('click', (e) => {
      if (!audio.duration) return;
      const rect = progressWrap.getBoundingClientRect();
      const pct = (e.clientX - rect.left) / rect.width;
      audio.currentTime = pct * audio.duration;
    });

    setPlaybackUi();
    updateTimeDisplay();
  });
}

function wireVideoPlayers() {
  document.querySelectorAll('[data-video-player]').forEach((container) => {
    const video = container.querySelector('[data-video-src]');
    const source = video?.querySelector('source[data-src]');
    const loadButton = container.querySelector('[data-video-load]');
    if (!video || !source || video.dataset.videoWired === 'true') return;

    video.dataset.videoWired = 'true';

    const loadVideo = async (autoplay = false) => {
      if (!source.getAttribute('src')) {
        source.setAttribute('src', source.dataset.src || video.dataset.videoSrc || '');
        video.load();
      }
      container.classList.add('is-loaded');
      if (!autoplay) return;
      try {
        await video.play();
      } catch (error) {
        console.warn('Video playback failed', error);
      }
    };

    loadButton?.addEventListener('click', () => {
      loadVideo(true);
    });

    video.addEventListener('play', () => {
      container.classList.add('is-loaded');
    });
  });
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}
