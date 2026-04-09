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
    { threshold: 0.12, rootMargin: '0px 0px -6% 0px' },
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

function init() {
  document.body.classList.add('js-ready');
  markCurrentNav();
  wireScrollState();
  wireRevealAnimations();
  wireCursorBeacon();
  wireMascot();
  wireButtons();
  wireCardGlow();
  renderMath();
  wireCodeBlocks();
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}
