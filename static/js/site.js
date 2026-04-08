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
  wireButtons();
  renderMath();
  wireCodeBlocks();
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}
