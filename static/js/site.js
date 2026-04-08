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

function randomRange(min, max) {
  return min + Math.random() * (max - min);
}

function easeInOutSine(value) {
  return -(Math.cos(Math.PI * value) - 1) / 2;
}

function wireHeroParticles() {
  const particleSurfaces = document.querySelectorAll('[data-particle-field]');
  if (!particleSurfaces.length) return;

  particleSurfaces.forEach((particleSurface) => {
    const stage = particleSurface.closest('.hero-particle-panel') ?? particleSurface.parentElement ?? particleSurface;
    const canvas = particleSurface.querySelector('.hero-particle-canvas');
    const context = canvas?.getContext?.('2d');
    if (!canvas || !context) return;

    const state = {
      particles: [],
      width: 0,
      height: 0,
      dpr: 1,
      pointer: { x: 0, y: 0, inside: false },
      clickImpulse: 0,
      ambientBurst: null,
      nextAmbientBurstAt: 0,
      rafId: 0,
    };

    const densityBias = (u, v) => clamp(0.14 + Math.pow(u * 0.62 + v * 0.38, 2.15), 0.1, 1);

    const center = () => ({ x: state.width * 0.68, y: state.height * 0.44 });

    const makeScatterVector = (u, v, intensity = 1) => {
      const dx = u - 0.68;
      const dy = v - 0.44;
      const distance = Math.max(0.02, Math.hypot(dx, dy));
      const radialX = dx / distance;
      const radialY = dy / distance;
      const tangentX = -radialY;
      const tangentY = radialX;
      return {
        x:
          (radialX * randomRange(0.045, 0.1) + tangentX * randomRange(-0.03, 0.03)) * intensity,
        y:
          (radialY * randomRange(0.05, 0.12) + tangentY * randomRange(-0.02, 0.02)) * intensity,
      };
    };

    const createParticle = ({ u, v, role = 'ring', accent = false }) => {
      const scatter = makeScatterVector(u, v, role === 'satellite' ? 1.15 : role === 'inner' ? 0.9 : 1);
      return {
        u,
        v,
        x: 0,
        y: 0,
        vx: 0,
        vy: 0,
        scatterU: scatter.x,
        scatterV: scatter.y,
        driftRadius: role === 'satellite' ? randomRange(8, 18) : randomRange(5, 12),
        driftPhase: randomRange(0, Math.PI * 2),
        driftSpeed: randomRange(0.55, 1.05),
        radius: accent ? randomRange(5.2, 8.2) : randomRange(3, 5.6),
        alpha: accent ? randomRange(0.82, 0.98) : randomRange(0.56, 0.88),
        color: accent ? '110, 231, 183' : Math.random() > 0.38 ? '246, 80, 255' : '193, 71, 255',
        shadow: accent ? randomRange(18, 28) : randomRange(12, 22),
      };
    };

    const buildFormation = () => {
      const particles = [];
      const ringTarget = 88;
      let guard = 0;

      while (particles.length < ringTarget && guard < 4000) {
        guard += 1;
        const angle = Math.random() * Math.PI * 2;
        const ringU = 0.68 + Math.cos(angle) * 0.23 * randomRange(0.88, 1.08);
        const ringV = 0.44 + Math.sin(angle) * 0.3 * randomRange(0.84, 1.12);
        if (ringU < 0.12 || ringU > 0.95 || ringV < 0.06 || ringV > 0.94) continue;
        if (Math.random() > densityBias(ringU, ringV)) continue;
        particles.push(
          createParticle({
            u: ringU,
            v: ringV,
            role: 'ring',
            accent: Math.random() < 0.12,
          }),
        );
      }

      for (let index = 0; index < 9; index += 1) {
        const angle = randomRange(0, Math.PI * 2);
        const distance = randomRange(0.07, 0.18);
        const innerU = 0.68 + Math.cos(angle) * distance;
        const innerV = 0.44 + Math.sin(angle) * distance * 1.15;
        particles.push(
          createParticle({
            u: innerU,
            v: innerV,
            role: 'inner',
            accent: Math.random() < 0.18,
          }),
        );
      }

      for (let index = 0; index < 14; index += 1) {
        let satelliteU = 0.68;
        let satelliteV = 0.44;
        let satelliteGuard = 0;
        while (satelliteGuard < 120) {
          satelliteGuard += 1;
          const angle = randomRange(0, Math.PI * 2);
          const haloScale = randomRange(1.18, 1.55);
          satelliteU = 0.68 + Math.cos(angle) * 0.23 * haloScale + randomRange(-0.015, 0.015);
          satelliteV = 0.44 + Math.sin(angle) * 0.3 * haloScale + randomRange(-0.02, 0.02);
          if (satelliteU >= 0.06 && satelliteU <= 0.97 && satelliteV >= 0.04 && satelliteV <= 0.96) {
            break;
          }
        }
        particles.push(
          createParticle({
            u: satelliteU,
            v: satelliteV,
            role: 'satellite',
            accent: Math.random() < 0.2,
          }),
        );
      }

      state.particles = particles;
    };

    const resize = () => {
      const rect = particleSurface.getBoundingClientRect();
      state.width = Math.max(1, rect.width);
      state.height = Math.max(1, rect.height);
      state.dpr = Math.min(window.devicePixelRatio || 1, 2);
      canvas.width = Math.round(state.width * state.dpr);
      canvas.height = Math.round(state.height * state.dpr);
      context.setTransform(state.dpr, 0, 0, state.dpr, 0, 0);
      if (!state.particles.length) {
        buildFormation();
      }
      state.particles.forEach((particle) => {
        particle.x = particle.u * state.width;
        particle.y = particle.v * state.height;
        particle.vx = 0;
        particle.vy = 0;
      });
      render(performance.now());
    };

    const scheduleAmbientBurst = (now = performance.now()) => {
      state.nextAmbientBurstAt = now + randomRange(2100, 3800);
    };

    const ambientEnvelope = (now) => {
      if (!state.ambientBurst) return 0;
      const progress = (now - state.ambientBurst.start) / state.ambientBurst.duration;
      if (progress >= 1) {
        state.ambientBurst = null;
        return 0;
      }
      return progress < 0.5
        ? easeInOutSine(progress / 0.5)
        : 1 - easeInOutSine((progress - 0.5) / 0.5);
    };

    const render = (now) => {
      context.clearRect(0, 0, state.width, state.height);
      const centerPoint = center();
      const envelope = ambientEnvelope(now);
      const scatterScale = Math.min(state.width, state.height);

      if (!state.pointer.inside && !state.ambientBurst && now >= state.nextAmbientBurstAt) {
        state.ambientBurst = {
          start: now,
          duration: randomRange(1100, 1650),
          strength: randomRange(0.56, 0.92),
        };
        scheduleAmbientBurst(now);
      }

      state.clickImpulse *= 0.92;

      state.particles.forEach((particle) => {
        const driftX = Math.cos(now * 0.001 * particle.driftSpeed + particle.driftPhase) * particle.driftRadius;
        const driftY =
          Math.sin(now * 0.00115 * particle.driftSpeed + particle.driftPhase * 1.3) * particle.driftRadius * 0.85;
        const burstX =
          particle.scatterU * scatterScale * envelope * (state.ambientBurst?.strength ?? 0.75);
        const burstY =
          particle.scatterV * scatterScale * envelope * (state.ambientBurst?.strength ?? 0.75);
        const targetX = particle.u * state.width + driftX + burstX;
        const targetY = particle.v * state.height + driftY + burstY;

        particle.vx += (targetX - particle.x) * 0.0085;
        particle.vy += (targetY - particle.y) * 0.0085;

        if (state.pointer.inside) {
          const dx = particle.x - state.pointer.x;
          const dy = particle.y - state.pointer.y;
          const distance = Math.max(0.001, Math.hypot(dx, dy));
          const radius = Math.min(state.width, state.height) * (0.2 + state.clickImpulse * 0.14);
          if (distance < radius) {
            const power = 1 - distance / radius;
            const impulse = 0.78 + state.clickImpulse * 1.9;
            particle.vx += (dx / distance) * power * impulse;
            particle.vy += (dy / distance) * power * impulse;
          }
        }

        const centerPullX = (particle.x - centerPoint.x) * 0.00018;
        const centerPullY = (particle.y - centerPoint.y) * 0.00018;
        particle.vx -= centerPullX;
        particle.vy -= centerPullY;

        particle.vx *= 0.93;
        particle.vy *= 0.93;
        particle.x += particle.vx;
        particle.y += particle.vy;

        context.beginPath();
        context.fillStyle = `rgba(${particle.color}, ${particle.alpha})`;
        context.shadowBlur = particle.shadow;
        context.shadowColor = `rgba(${particle.color}, 0.36)`;
        context.arc(particle.x, particle.y, particle.radius, 0, Math.PI * 2);
        context.fill();
      });

      context.shadowBlur = 0;
    };

    if (prefersReducedMotion()) {
      buildFormation();
      resize();
      scheduleAmbientBurst();
      return;
    }

    const pointerFromEvent = (event) => {
      const rect = particleSurface.getBoundingClientRect();
      state.pointer.x = event.clientX - rect.left;
      state.pointer.y = event.clientY - rect.top;
      state.pointer.inside = true;
    };

    const tick = (now) => {
      render(now);
      state.rafId = window.requestAnimationFrame(tick);
    };

    buildFormation();
    resize();
    scheduleAmbientBurst();
    state.rafId = window.requestAnimationFrame(tick);

    stage.addEventListener('pointerenter', (event) => {
      pointerFromEvent(event);
    });

    stage.addEventListener('pointermove', (event) => {
      pointerFromEvent(event);
    });

    stage.addEventListener('pointerleave', () => {
      state.pointer.inside = false;
      scheduleAmbientBurst(performance.now() - 900);
    });

    stage.addEventListener('pointerdown', (event) => {
      pointerFromEvent(event);
      state.clickImpulse = Math.max(state.clickImpulse, 1);
    });

    if (typeof ResizeObserver !== 'undefined') {
      const observer = new ResizeObserver(() => resize());
      observer.observe(stage);
    } else {
      window.addEventListener('resize', resize);
    }
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
  wireHeroParticles();
  wireButtons();
  renderMath();
  wireCodeBlocks();
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}
