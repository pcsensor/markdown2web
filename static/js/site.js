const reduceMotionQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
const finePointerQuery = window.matchMedia('(pointer: fine)');
const prefersReducedMotion = () => reduceMotionQuery.matches;
const hasFinePointer = () => finePointerQuery.matches;
let turnstileScriptPromise = null;

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
    if (element.hasAttribute('data-static-button')) return;
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
  const csrfToken = article.dataset.csrfToken || '';
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
        'X-CSRF-Token': csrfToken,
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
      headers: { Accept: 'application/json', 'X-CSRF-Token': csrfToken },
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
  wireLazyTurnstile();
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

function wireLazyTurnstile() {
  document.querySelectorAll('[data-turnstile-form]').forEach((form) => {
    const container = form.querySelector('[data-turnstile-lazy]');
    const responseInput = form.querySelector('[data-turnstile-response]');
    const submitButton = form.querySelector('button[type="submit"]');
    if (!container || !responseInput || form.dataset.turnstileWired === 'true') return;

    form.dataset.turnstileWired = 'true';
    let widgetId = null;
    let pending = false;

    const setSubmitting = (submitting) => {
      pending = submitting;
      if (submitButton) {
        submitButton.disabled = submitting;
        submitButton.textContent = submitting ? '正在验证…' : submitButton.dataset.originalText;
      }
    };

    if (submitButton) {
      submitButton.dataset.originalText = submitButton.textContent;
    }

    const removeDuplicateResponseFields = () => {
      form
        .querySelectorAll('input[name="cf-turnstile-response"]:not([data-turnstile-response])')
        .forEach((field) => field.remove());
    };

    const renderWidget = () => {
      if (widgetId !== null) return true;
      if (!window.turnstile) return false;

      widgetId = window.turnstile.render(container, {
        sitekey: container.dataset.sitekey,
        theme: container.dataset.theme || 'light',
        execution: 'execute',
        'response-field': false,
        callback(token) {
          responseInput.value = token;
          removeDuplicateResponseFields();
          setSubmitting(false);
          HTMLFormElement.prototype.submit.call(form);
        },
        'error-callback'() {
          responseInput.value = '';
          setSubmitting(false);
          container.dataset.turnstileError = 'true';
        },
        'expired-callback'() {
          responseInput.value = '';
        },
      });

      return widgetId !== undefined && widgetId !== null;
    };

    form.addEventListener('submit', async (event) => {
      if (responseInput.value) {
        removeDuplicateResponseFields();
        return;
      }
      if (!form.checkValidity()) return;

      event.preventDefault();
      if (pending) return;

      setSubmitting(true);
      try {
        await loadTurnstileScript();
      } catch (error) {
        console.warn('Turnstile script failed to load', error);
        container.textContent = '人机验证加载失败，请刷新页面后重试。';
        setSubmitting(false);
        return;
      }

      if (!renderWidget()) {
        container.textContent = '人机验证初始化失败，请刷新页面后重试。';
        setSubmitting(false);
        return;
      }

      responseInput.value = '';
      container.dataset.turnstileError = 'false';
      window.turnstile.execute(widgetId);
    });
  });
}

function loadTurnstileScript() {
  if (window.turnstile) {
    return Promise.resolve(window.turnstile);
  }
  if (turnstileScriptPromise) {
    return turnstileScriptPromise;
  }

  turnstileScriptPromise = new Promise((resolve, reject) => {
    const script = document.createElement('script');
    script.src = 'https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit';
    script.async = true;
    script.defer = true;
    script.crossOrigin = 'anonymous';
    script.onload = () => {
      if (window.turnstile) {
        resolve(window.turnstile);
      } else {
        reject(new Error('Turnstile unavailable after script load'));
      }
    };
    script.onerror = () => reject(new Error('Turnstile script load failed'));
    document.head.appendChild(script);
  });

  return turnstileScriptPromise;
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
    const toggleButton = container.querySelector('[data-video-toggle]');
    const progress = container.querySelector('[data-video-progress]');
    const progressFill = container.querySelector('[data-video-progress-fill]');
    const timeDisplay = container.querySelector('[data-video-time]');
    const volumeInput = container.querySelector('[data-video-volume]');
    const volumeLabel = container.querySelector('[data-video-volume-label]');
    const fullscreenButton = container.querySelector('[data-video-fullscreen]');
    const speedSelect = container.querySelector('[data-video-speed]');
    const danmakuSizeSelect = container.querySelector('[data-video-danmaku-size]');
    const danmakuLayer = container.querySelector('[data-video-danmaku-layer]');
    const danmakuForm = container.querySelector('[data-video-danmaku-form]');
    const danmakuInput = container.querySelector('[data-video-danmaku-input]');
    const danmakuColor = container.querySelector('[data-video-danmaku-color]');
    const danmakuLogin = container.querySelector('[data-video-danmaku-login]');
    const article = container.closest('[data-note-article]');
    if (!video || !source || video.dataset.videoWired === 'true') return;

    video.dataset.videoWired = 'true';
    const noteSlug = article?.dataset.noteSlug || '';
    const accountUrl = article?.dataset.accountUrl || '/account';
    const danmakuEnabled = article?.dataset.annotationEnabled === 'true';
    const videoKey = video.dataset.videoKey || video.dataset.videoSrc || '';
    const danmakuState = {
      items: [],
      shown: new Set(),
      lastTimeMs: 0,
      loaded: false,
      seeking: false,
    };
    let controlsTimer = 0;

    const formatTime = (sec) => {
      if (isNaN(sec) || !isFinite(sec)) return '00:00';
      const m = Math.floor(sec / 60);
      const s = Math.floor(sec % 60);
      return `${String(m).padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
    };

    const updateTime = () => {
      if (timeDisplay) {
        timeDisplay.textContent = `${formatTime(video.currentTime)}/${formatTime(video.duration)}`;
      }
      if (progressFill) {
        const pct = video.duration ? (video.currentTime / video.duration) * 100 : 0;
        progressFill.style.width = `${Math.min(100, Math.max(0, pct))}%`;
      }
    };

    const updateControls = () => {
      const playing = !video.paused && !video.ended;
      container.classList.toggle('is-playing', playing);
      if (toggleButton) {
        toggleButton.textContent = playing ? 'Ⅱ' : '▶';
        toggleButton.setAttribute('aria-label', playing ? '暂停' : '播放');
      }
      if (volumeInput) {
        volumeInput.value = video.muted ? '0' : String(video.volume);
      }
      if (volumeLabel) {
        const pct = video.muted ? 0 : Math.round(video.volume * 100);
        volumeLabel.textContent = `${pct}%`;
      }
      updateTime();
    };

    const showControls = (sticky = false) => {
      container.classList.add('is-controls-visible');
      window.clearTimeout(controlsTimer);
      
      // 如果视频正在播放且不是强制常驻模式，开启自动隐藏定时器
      if (!sticky && !video.paused && !video.ended) {
        controlsTimer = window.setTimeout(() => {
          if (!container.matches(':focus-within')) {
            container.classList.remove('is-controls-visible');
          }
        }, 2500);
      }
    };

    const showDanmaku = (item) => {
      if (!danmakuLayer || !item?.body) return;
      const node = document.createElement('span');
      node.className = 'video-danmaku-item';
      node.textContent = item.body;
      node.style.color = item.color || '#fff';
      // 将弹幕轨道限制在视频上方的 40% 区域
      const lane = Math.abs(Number(item.id || item.time_ms || 0)) % 5;
      node.style.setProperty('--danmaku-top', `${5 + lane * 7}%`);
      danmakuLayer.appendChild(node);
      node.addEventListener('animationend', () => node.remove(), { once: true });
    };

    const syncDanmaku = (forceBaseline = false) => {
      if (!danmakuState.loaded) return;
      const currentMs = Math.floor(video.currentTime * 1000);
      if (forceBaseline || danmakuState.seeking || Math.abs(currentMs - danmakuState.lastTimeMs) > 1800) {
        danmakuState.shown.clear();
        danmakuState.lastTimeMs = currentMs;
        danmakuState.seeking = false;
        return;
      }
      const startMs = Math.min(danmakuState.lastTimeMs, currentMs);
      const endMs = Math.max(danmakuState.lastTimeMs, currentMs) + 250;
      danmakuState.items.forEach((item) => {
        if (danmakuState.shown.has(item.id)) return;
        if (item.time_ms >= startMs && item.time_ms <= endMs) {
          danmakuState.shown.add(item.id);
          showDanmaku(item);
        }
      });
      danmakuState.lastTimeMs = currentMs;
    };

    const loadDanmaku = async () => {
      // 移除 danmakuEnabled 限制，让所有人都能加载并看到弹幕
      if (danmakuState.loaded || !noteSlug || !videoKey) return;
      const params = new URLSearchParams({ video_src: videoKey });
      const response = await fetch(`/api/notes/${encodeURIComponent(noteSlug)}/danmaku?${params}`, {
        headers: { Accept: 'application/json' },
      });
      if (!response.ok) return;
      const data = await response.json();
      danmakuState.items = data.danmaku || [];
      danmakuState.loaded = true;
      syncDanmaku(true);
    };

    const loadVideo = async (autoplay = false) => {
      if (!source.getAttribute('src')) {
        source.setAttribute('src', source.dataset.src || video.dataset.videoSrc || '');
        video.load();
      }
      container.classList.add('is-loaded');
      showControls(!autoplay);
      
      // 即使不自动播放，也应该尝试加载弹幕，以便用户点击播放时能看到
      await loadDanmaku();

      if (!autoplay) return;
      try {
        await video.play();
        updateControls();
      } catch (error) {
        console.warn('Video playback failed', error);
      }
    };

    loadButton?.addEventListener('click', () => {
      loadVideo(true);
      // 点击播放后自动聚焦到容器，以便立即使用键盘控制
      container.focus();
    });

    toggleButton?.addEventListener('click', async () => {
      await loadVideo(false);
      if (video.paused || video.ended) {
        try {
          await video.play();
        } catch (error) {
          console.warn('Video playback failed', error);
        }
      } else {
        video.pause();
      }
      updateControls();
    });

    video.addEventListener('click', async () => {
      await loadVideo(false);
      if (video.paused || video.ended) {
        await video.play().catch((error) => console.warn('Video playback failed', error));
      } else {
        video.pause();
      }
      updateControls();
    });

    video.addEventListener('play', () => {
      container.classList.add('is-loaded');
      loadDanmaku();
      showControls();
      updateControls();
    });

    video.addEventListener('pause', () => {
      showControls(true);
      updateControls();
    });
    video.addEventListener('ended', () => {
      showControls(true);
      updateControls();
    });
    video.addEventListener('loadedmetadata', updateControls);
    video.addEventListener('seeking', () => {
      danmakuState.seeking = true;
      danmakuState.lastTimeMs = Math.floor(video.currentTime * 1000);
      danmakuLayer?.querySelectorAll('.video-danmaku-item').forEach((item) => item.remove());
    });
    video.addEventListener('seeked', () => {
      syncDanmaku(true);
    });
    video.addEventListener('timeupdate', () => {
      updateTime();
      syncDanmaku();
    });
    video.addEventListener('volumechange', updateControls);

    progress?.addEventListener('click', async (event) => {
      await loadVideo(false);
      if (!video.duration) return;
      const rect = progress.getBoundingClientRect();
      const pct = (event.clientX - rect.left) / rect.width;
      video.currentTime = Math.min(video.duration, Math.max(0, pct * video.duration));
      syncDanmaku(true);
      updateTime();
    });

    volumeInput?.addEventListener('input', () => {
      const volume = Number(volumeInput.value);
      video.volume = Number.isFinite(volume) ? Math.min(1, Math.max(0, volume)) : 1;
      video.muted = video.volume === 0;
      updateControls();
    });

    speedSelect?.addEventListener('change', () => {
      const rate = Number(speedSelect.value);
      video.playbackRate = Number.isFinite(rate) && rate > 0 ? rate : 1;
    });

    danmakuSizeSelect?.addEventListener('change', () => {
      container.style.setProperty('--danmaku-font-size', danmakuSizeSelect.value || '1.25rem');
    });
    container.style.setProperty('--danmaku-font-size', danmakuSizeSelect?.value || '1.25rem');

    // 记录进入全屏前的滚动位置
    let preFullscreenScrollY = 0;

    fullscreenButton?.addEventListener('click', async () => {
      // 1. 尝试退出全屏
      if (document.fullscreenElement || document.webkitFullscreenElement) {
        if (document.exitFullscreen) await document.exitFullscreen().catch(() => {});
        else if (document.webkitExitFullscreen) document.webkitExitFullscreen();
        return;
      }
      
      preFullscreenScrollY = window.scrollY;
      
      // 2. 针对 iPhone/iOS 的特殊处理：它们通常不支持容器全屏，只能让 video 本身全屏
      if (!container.requestFullscreen && video.webkitEnterFullscreen) {
        video.webkitEnterFullscreen();
        return;
      }

      // 3. 标准全屏请求 (Android/Desktop)
      const requestFs = container.requestFullscreen || container.webkitRequestFullscreen || container.mozRequestFullScreen || container.msRequestFullscreen;
      if (requestFs) {
        await requestFs.call(container).catch((err) => {
          console.warn('Fullscreen request failed', err);
          // 最后的保底：尝试视频全屏
          if (video.webkitEnterFullscreen) video.webkitEnterFullscreen();
        });
      }
    });

    // 针对 iOS 的全屏状态监听
    video.addEventListener('webkitbeginfullscreen', () => {
      container.classList.add('is-controls-visible');
    });
    video.addEventListener('webkitendfullscreen', () => {
      if (preFullscreenScrollY >= 0) {
        window.scrollTo(0, preFullscreenScrollY);
      }
    });

    // 监听全屏状态变化，处理退出后的复位
    container.addEventListener('fullscreenchange', () => {
      if (!document.fullscreenElement) {
        // 1. 退出时立即让容器失焦，防止浏览器尝试自动“滚动到焦点”导致跳动
        if (document.activeElement === container) {
          container.blur();
        }
        
        // 2. 多阶段强制复位坐标，应对文档高度动态变化的情况
        const restore = () => {
          if (typeof preFullscreenScrollY === 'number') {
            window.scrollTo({
              top: preFullscreenScrollY,
              behavior: 'instant'
            });
          }
        };

        // 立即复位一次，并分别在布局可能变动的时间点追加复位，确保彻底解决“底部截断”问题
        restore();
        setTimeout(restore, 20);
        setTimeout(restore, 100);
        setTimeout(restore, 250);
      }
    });

    container.addEventListener('pointermove', () => {
      showControls();
    });
    container.addEventListener('pointerleave', () => {
      if (!video.paused && !video.ended) {
        window.clearTimeout(controlsTimer);
        container.classList.remove('is-controls-visible');
      }
    });

    // 允许容器接收焦点以支持键盘控制
    container.tabIndex = 0;
    container.addEventListener('keydown', (e) => {
      // 如果正在输入文字，不触发快捷键
      if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA' || e.target.tagName === 'SELECT') return;

      const step = 3; // 前进/后退步长 3s
      switch (e.key) {
        case ' ':
          e.preventDefault();
          if (video.paused) video.play(); else video.pause();
          showControls();
          break;
        case 'ArrowLeft':
          e.preventDefault();
          video.currentTime = Math.max(0, video.currentTime - step);
          showControls();
          break;
        case 'ArrowRight':
          e.preventDefault();
          video.currentTime = Math.min(video.duration, video.currentTime + step);
          showControls();
          break;
      }
    });

    if (danmakuEnabled) {
      danmakuForm?.classList.add('is-enabled');
      if (danmakuInput) danmakuInput.placeholder = '发送弹幕';
      if (danmakuLogin) danmakuLogin.hidden = true;
    } else {
      if (danmakuForm) danmakuForm.hidden = true;
      if (danmakuLogin) danmakuLogin.href = accountUrl;
    }

    danmakuForm?.addEventListener('submit', async (event) => {
      event.preventDefault();
      if (!danmakuEnabled || !danmakuInput || !noteSlug || !videoKey) return;
      const body = danmakuInput.value.trim();
      if (!body) return;
      const payload = {
        video_src: videoKey,
        time_ms: Math.max(0, Math.floor(video.currentTime * 1000)),
        body,
        color: danmakuColor?.value || '#ffffff',
      };
      const response = await fetch(`/api/notes/${encodeURIComponent(noteSlug)}/danmaku`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Accept: 'application/json',
          'X-CSRF-Token': csrfToken,
        },
        body: JSON.stringify(payload),
      });
      if (response.status === 401) {
        window.location.href = accountUrl;
        return;
      }
      if (!response.ok) return;
      const created = await response.json();
      danmakuInput.value = '';
      danmakuState.items.push(created);
      danmakuState.shown.add(created.id);
      showDanmaku(created);
    });

    updateControls();
  });
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}
