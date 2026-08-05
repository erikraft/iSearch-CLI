/* ============================================================
   ERIKRAFT — PREMIUM INTERACTIVE CORE
   ============================================================ */

document.addEventListener("DOMContentLoaded", () => {
    // 1. Initializations
    initLoader();
    initHeaderScroll();
    initMobileMenu();
    initThreeBg();
    initTyping();
    initScrollReveal();
    updateDynamicTargets();
    initCounters();
    initScrollProgress();
    initCliCarousel(); // Modern Swerdlow Brainless Carousel replacing basic simulator
});

/**
 * Site Loader Handler
 */
function initLoader() {
    const loader = document.querySelector('.site-loader');
    if (!loader) return;

    // Wait until window load or at least 1 second to feel smooth
    window.addEventListener('load', () => {
        setTimeout(() => {
            loader.classList.add('hidden');
            document.body.classList.remove('loading');

            // Trigger Hero entry animations via GSAP
            if (typeof gsap !== 'undefined') {
                gsap.from('.hero-content > *', {
                    opacity: 0,
                    y: 40,
                    duration: 1.2,
                    stagger: 0.15,
                    ease: "power4.out"
                });

                gsap.from('.hero-photo-img', {
                    opacity: 0,
                    y: 100,
                    scale: 0.9,
                    duration: 1.6,
                    ease: "power3.out",
                    delay: 0.4
                });

                // Smooth continuous marquee text translation from right to left
                const marqueeInner = document.querySelector('.hero-marquee-inner');
                if (marqueeInner) {
                    // Duplicate text content dynamically to ensure seamless continuous sliding loop
                    const marqueeText = marqueeInner.textContent;
                    marqueeInner.textContent = marqueeText + ' ' + marqueeText;

                    gsap.to(marqueeInner, {
                        xPercent: -50,
                        ease: "none",
                        duration: 35,
                        repeat: -1
                    });
                }

                // Discretely follow mouse coordinates with parallax depth on the central image
                window.addEventListener('mousemove', (e) => {
                    const pctX = (e.clientX - window.innerWidth / 2) / (window.innerWidth / 2);
                    const pctY = (e.clientY - window.innerHeight / 2) / (window.innerHeight / 2);
                    gsap.to('.hero-photo-img', {
                        x: pctX * 15,
                        y: pctY * 10,
                        rotationY: pctX * 5,
                        rotationX: -pctY * 5,
                        duration: 0.8,
                        ease: "power1.out"
                    });
                });
            }
        }, 600);
    });
}

/**
 * Header Scroll Shrink & Blur
 */
function initHeaderScroll() {
    const header = document.querySelector('.site-header');
    if (!header) return;

    window.addEventListener('scroll', () => {
        if (window.scrollY > 30) {
            header.classList.add('scrolled');
        } else {
            header.classList.remove('scrolled');
        }
    });
}

/**
 * Favicon Blinking Animation
 * Keeps the static favicon as canonical for bots/indexing,
 * and animates dynamic favicon on standard browsers after loading.
 */
function initFaviconAnimation() {
    const faviconLinks = document.querySelectorAll("link[rel*='icon']");
    if (faviconLinks.length === 0) return;

    const faviconOpen = '/favicon.ico';
    const faviconClosed = '/favicon2.ico';

    // Preload both favicons to prevent flicker / dynamic network delay during switching
    const preloadOpen = new Image();
    preloadOpen.src = faviconOpen;
    const preloadClosed = new Image();
    preloadClosed.src = faviconClosed;

    let isClosed = false;

    function tick() {
        const nextSrc = isClosed ? faviconOpen : faviconClosed;
        isClosed = !isClosed;

        faviconLinks.forEach(link => {
            if (link.getAttribute('href')) {
                link.setAttribute('href', nextSrc);
            }
        });

        // Set natural-feeling blink intervals (800ms open, 150ms blinking/closed)
        const delay = isClosed ? 150 : 800;
        setTimeout(tick, delay);
    }

    // Start tick cycle after an initial open state delay
    setTimeout(tick, 800);
}

// Start favicon animation only after full window load to ensure search engines and bots
// have crawled the original, static canonical favicon page layout completely.
window.addEventListener('load', initFaviconAnimation);

/**
 * Mobile Navigation Toggle (with accessibility)
 */
function initMobileMenu() {
    const toggle = document.querySelector('.menu-toggle');
    const nav = document.querySelector('.site-nav');

    if (!toggle || !nav) return;

    toggle.addEventListener('click', () => {
        const isOpen = nav.classList.toggle('is-open');
        toggle.classList.toggle('is-active');
        toggle.setAttribute('aria-expanded', isOpen);

        if (isOpen) {
            document.body.classList.add('menu-open');
        } else {
            document.body.classList.remove('menu-open');
        }
    });

    // Close menu on navigation click
    nav.querySelectorAll('a').forEach(link => {
        link.addEventListener('click', () => {
            nav.classList.remove('is-open');
            toggle.classList.remove('is-active');
            toggle.setAttribute('aria-expanded', 'false');
            document.body.classList.remove('menu-open');
        });
    });
}

/**
 * Three.js Interactive Particles Background (Low GPU impact)
 */
function initThreeBg() {
    const canvas = document.getElementById('three-bg');
    if (!canvas || typeof THREE === 'undefined') return;

    const renderer = new THREE.WebGLRenderer({ canvas, alpha: true, antialias: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setSize(window.innerWidth, window.innerHeight);

    const scene = new THREE.Scene();

    // Camera
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 100);
    camera.position.z = 30;

    // Particles Geometry
    const particlesCount = window.innerWidth < 768 ? 150 : 300;
    const geometry = new THREE.BufferGeometry();
    const positions = new Float32Array(particlesCount * 3);
    const colors = new Float32Array(particlesCount * 3);

    // Color choices: Blue, Purple, Cyan, White/Beige
    const palette = [
        new THREE.Color('#0077ff'), // Blue
        new THREE.Color('#8b5cf6'), // Purple
        new THREE.Color('#06b6d4'), // Cyan
        new THREE.Color('#c5bba8')  // Beige
    ];

    for (let i = 0; i < particlesCount; i++) {
        // Distribute coordinates in a 3D sphere/cube
        positions[i * 3] = (Math.random() - 0.5) * 60;
        positions[i * 3 + 1] = (Math.random() - 0.5) * 60;
        positions[i * 3 + 2] = (Math.random() - 0.5) * 60;

        // Colors
        const chosenColor = palette[Math.floor(Math.random() * palette.length)];
        colors[i * 3] = chosenColor.r;
        colors[i * 3 + 1] = chosenColor.g;
        colors[i * 3 + 2] = chosenColor.b;
    }

    geometry.setAttribute('position', new THREE.BufferAttribute(positions, 3));
    geometry.setAttribute('color', new THREE.BufferAttribute(colors, 3));

    // Material with transparent round particle simulation
    // Using HTML Canvas generated texture for a rounded glowing dot
    const pCanvas = document.createElement('canvas');
    pCanvas.width = 16;
    pCanvas.height = 16;
    const ctx = pCanvas.getContext('2d');
    const grad = ctx.createRadialGradient(8, 8, 0, 8, 8, 8);
    grad.addColorStop(0, 'rgba(255, 255, 255, 1)');
    grad.addColorStop(1, 'rgba(255, 255, 255, 0)');
    ctx.fillStyle = grad;
    ctx.fillRect(0, 0, 16, 16);

    const texture = new THREE.CanvasTexture(pCanvas);

    const material = new THREE.PointsMaterial({
        size: 0.6,
        sizeAttenuation: true,
        vertexColors: true,
        transparent: true,
        opacity: 0.75,
        map: texture,
        depthWrite: false,
        blending: THREE.AdditiveBlending
    });

    const particles = new THREE.Points(geometry, material);
    scene.add(particles);

    // Mouse movement response
    let mouseX = 0;
    let mouseY = 0;
    let targetX = 0;
    let targetY = 0;

    window.addEventListener('mousemove', (e) => {
        mouseX = (e.clientX - window.innerWidth / 2) / 100;
        mouseY = (e.clientY - window.innerHeight / 2) / 100;
    });

    // Handle Resize
    window.addEventListener('resize', () => {
        camera.aspect = window.innerWidth / window.innerHeight;
        camera.updateProjectionMatrix();
        renderer.setSize(window.innerWidth, window.innerHeight);
    });

    // Animation Loop
    const clock = new THREE.Clock();

    function animate() {
        requestAnimationFrame(animate);

        const elapsedTime = clock.getElapsedTime();

        // Slow rotate
        particles.rotation.y = elapsedTime * 0.02;
        particles.rotation.x = elapsedTime * 0.01;

        // Discretely follow mouse coords
        targetX += (mouseX - targetX) * 0.05;
        targetY += (mouseY - targetY) * 0.05;

        particles.position.x = targetX * 1.5;
        particles.position.y = -targetY * 1.5;

        renderer.render(scene, camera);
    }

    animate();
}

/**
 * Dynamic Core Targets (Birthdate & Years of Experience)
 */
function updateDynamicTargets() {
    const ageElement = document.getElementById('age-counter');
    const expElement = document.getElementById('exp-counter');
    const etecStatus = document.getElementById('etec-status');

    // Brasília Time (UTC-3)
    const now = new Date();
    const brTime = new Date(now.toLocaleString("en-US", {timeZone: "America/Sao_Paulo"}));

    const currentYear = brTime.getFullYear();
    const currentMonth = brTime.getMonth() + 1;
    const currentDay = brTime.getDate();

    if (ageElement) {
        // Birthdate: July 22, 2009
        let age = currentYear - 2009;
        if (currentMonth < 7 || (currentMonth === 7 && currentDay < 22)) {
            age--;
        }
        ageElement.setAttribute('data-target', age);
    }

    if (expElement) {
        // Experience base start year: 2023
        let exp = currentYear - 2023;
        expElement.setAttribute('data-target', exp);
    }

    if (etecStatus) {
        // ETEC Course switches to Completed/Concluído in Dec 2028
        if (currentYear > 2028 || (currentYear === 2028 && currentMonth === 12)) {
            etecStatus.textContent = "Concluído 2028";
        } else {
            etecStatus.textContent = "Fev 2026 – Dez 2028";
        }
    }
}

/**
 * Typing Animation
 */
function initTyping() {
    const target = document.querySelector('.typing-text');
    if (!target) return;

    const phrases = [
        'Erik Rodrigues Balisa!',
        'Full Stack Developer!',
        'um Desenvolvedor do Futuro!',
        'ErikrafT!',
        'especialista em IA & Automação!',
        'um apaixonado por Clean Code!'
    ];

    let phraseIdx = 0;
    let charIdx = 0;
    let isDeleting = false;
    let typeSpeed = 100;

    function type() {
        const currentPhrase = phrases[phraseIdx];

        if (isDeleting) {
            target.textContent = currentPhrase.substring(0, charIdx - 1);
            charIdx--;
            typeSpeed = 50;
        } else {
            target.textContent = currentPhrase.substring(0, charIdx + 1);
            charIdx++;
            typeSpeed = 100;
        }

        if (!isDeleting && charIdx === currentPhrase.length) {
            isDeleting = true;
            typeSpeed = 2500; // Pause at the end
        } else if (isDeleting && charIdx === 0) {
            isDeleting = false;
            phraseIdx = (phraseIdx + 1) % phrases.length;
            typeSpeed = 500; // Pause before typing next phrase
        }

        setTimeout(type, typeSpeed);
    }

    type();
}

/**
 * Scroll Reveal Animation (Modern Intersection Observer)
 */
function initScrollReveal() {
    const reveals = document.querySelectorAll('.reveal');

    const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.classList.add('is-visible');

                // If element is a card, also stagger-animate some children via GSAP
                if (typeof gsap !== 'undefined') {
                    const cards = entry.target.querySelectorAll('.premium-card, .info-card, .showcase-card, .timeline-item');
                    if (cards.length > 0) {
                        gsap.from(cards, {
                            opacity: 0,
                            y: 20,
                            duration: 0.8,
                            stagger: 0.1,
                            ease: "power2.out"
                        });
                    }
                }
            }
        });
    }, {
        threshold: 0.1,
        rootMargin: "0px 0px -50px 0px"
    });

    reveals.forEach(el => observer.observe(el));
}

/**
 * Numeric Count-Up animations powered by GSAP
 */
function initCounters() {
    const counters = document.querySelectorAll('.counter');
    if (counters.length === 0 || typeof gsap === 'undefined') return;

    const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                const target = entry.target;
                const countTo = parseInt(target.getAttribute('data-target')) || 0;

                const obj = { val: 0 };
                gsap.to(obj, {
                    val: countTo,
                    duration: 2,
                    ease: "power3.out",
                    onUpdate: () => {
                        target.textContent = Math.floor(obj.val);
                    },
                    onComplete: () => {
                        target.textContent = countTo;
                    }
                });

                observer.unobserve(target);
            }
        });
    }, { threshold: 0.5 });

    counters.forEach(counter => observer.observe(counter));
}

/**
 * Scroll Progress Bar
 */
function initScrollProgress() {
    const bar = document.createElement('div');
    bar.style.position = 'fixed';
    bar.style.top = '0';
    bar.style.left = '0';
    bar.style.height = '3px';
    bar.style.background = 'linear-gradient(90deg, var(--accent), var(--accent-light))';
    bar.style.zIndex = '99999';
    bar.style.width = '0%';
    bar.style.pointerEvents = 'none';
    bar.style.boxShadow = '0 0 10px var(--accent-light)';

    document.body.appendChild(bar);

    window.addEventListener('scroll', () => {
        const scrolled = (window.scrollY / (document.documentElement.scrollHeight - window.innerHeight)) * 100;
        bar.style.width = scrolled + '%';
    });
}

/**
 * Modern High-Fidelity CLI Carousel inspired by Swerdlow (Brainless)
 * Uses GSAP ScrollTrigger to pin and horizontally slide terminal panels
 */
function initCliCarousel() {
    const section = document.getElementById('ai-agents-section');
    const container = document.querySelector('.cli-carousel-container');
    const track = document.querySelector('.cli-carousel-track');
    const panels = gsap.utils.toArray('.cli-panel');

    if (!section || !container || !track || panels.length < 3) return;
    if (typeof gsap === 'undefined' || typeof ScrollTrigger === 'undefined') return;

    // Register ScrollTrigger plugin safely
    gsap.registerPlugin(ScrollTrigger);

    // Refresh ScrollTrigger after fonts, images and layout settle
    window.addEventListener('load', () => {
        if (typeof ScrollTrigger !== 'undefined') {
            ScrollTrigger.refresh();
        }
    });

    window.addEventListener('resize', () => {
        if (typeof ScrollTrigger !== 'undefined') {
            ScrollTrigger.refresh();
        }
    });

    const calculateCliTrigger = () => {
        const sectionHeight = section.offsetHeight;
        const viewportHeight = window.innerHeight;
        const canFullyFit = sectionHeight <= viewportHeight;
        const startPoint = canFullyFit ? 'bottom bottom' : 'top top';
        const endDistance = Math.max(
            sectionHeight * 1.3,
            viewportHeight * 1.15,
            container.offsetWidth * 0.8
        );

        return {
            start: startPoint,
            end: `+=${Math.round(endDistance)}`
        };
    };

    // Initial state: Center the first card, push others to the right
    gsap.set(panels[0], { xPercent: -50, yPercent: -50, scale: 1, opacity: 1, filter: "blur(0px)", zIndex: 10 });
    gsap.set(panels[1], { xPercent: 120, yPercent: -50, scale: 0.9, opacity: 0, filter: "blur(4px)", zIndex: 5 });
    gsap.set(panels[2], { xPercent: 240, yPercent: -50, scale: 0.8, opacity: 0, filter: "blur(6px)", zIndex: 1 });

    // Progress Indicators Sync and Interaction Setup
    const dots = document.querySelectorAll('.indicator-dot');

    function updateActiveDot(index) {
        dots.forEach((dot, idx) => {
            if (idx === index) {
                dot.classList.add('active');
            } else {
                dot.classList.remove('active');
            }
        });
    }

    const mm = gsap.matchMedia();

    mm.add({ all: "(min-width: 0px)" }, () => {
        // Restore center positioning for pinning across all widths.
        gsap.set(panels, { clearProps: "all" });
        gsap.set(panels[0], { xPercent: -50, yPercent: -50, scale: 1, opacity: 1, filter: "blur(0px)", zIndex: 10 });
        gsap.set(panels[1], { xPercent: 120, yPercent: -50, scale: 0.9, opacity: 0, filter: "blur(4px)", zIndex: 5 });
        gsap.set(panels[2], { xPercent: 240, yPercent: -50, scale: 0.8, opacity: 0, filter: "blur(6px)", zIndex: 1 });

        const xOffset = Math.min(60, Math.max(40, window.innerWidth * 0.05));

        // Timeline synchronized with vertical scroll and full-section visibility.
        const tl = gsap.timeline({
            scrollTrigger: {
                trigger: section,
                id: 'cli-pin',
                start: () => calculateCliTrigger().start,
                end: () => calculateCliTrigger().end,
                pin: true,
                pinSpacing: true,
                scrub: 1,
                anticipatePin: 1,
                invalidateOnRefresh: true,
                onUpdate: (self) => {
                    const progress = self.progress;
                    if (progress < 0.33) {
                        updateActiveDot(0);
                    } else if (progress < 0.66) {
                        updateActiveDot(1);
                    } else {
                        updateActiveDot(2);
                    }
                }
            }
        });

        // Slide transitions between terminal panels.
        tl.to(panels[0], {
            xPercent: -50 - xOffset,
            scale: 0.93,
            opacity: 0.5,
            filter: "blur(2px)",
            zIndex: 5,
            duration: 1
        }, "scroll1")
        .to(panels[1], {
            xPercent: -50,
            scale: 1,
            opacity: 1,
            filter: "blur(0px)",
            zIndex: 10,
            duration: 1
        }, "scroll1")
        .to(panels[2], {
            xPercent: -50 + xOffset + 10,
            scale: 0.9,
            opacity: 0.15,
            filter: "blur(4px)",
            zIndex: 1,
            duration: 1
        }, "scroll1")
        .to(panels[0], {
            xPercent: -50 - xOffset - 10,
            scale: 0.85,
            opacity: 0.15,
            filter: "blur(4px)",
            zIndex: 1,
            duration: 1
        }, "scroll2")
        .to(panels[1], {
            xPercent: -50 - xOffset,
            scale: 0.93,
            opacity: 0.5,
            filter: "blur(2px)",
            zIndex: 5,
            duration: 1
        }, "scroll2")
        .to(panels[2], {
            xPercent: -50,
            scale: 1,
            opacity: 1,
            filter: "blur(0px)",
            zIndex: 10,
            duration: 1
        }, "scroll2");

        // Make dots clickable on desktop by smoothly scrolling to respective scroll progress offset.
        dots.forEach((dot, idx) => {
            dot.addEventListener('click', () => {
                const scrollTriggerInstance = ScrollTrigger.getById("cli-pin") || ScrollTrigger.getAll().find(st => st.trigger === section);
                if (scrollTriggerInstance) {
                    const start = scrollTriggerInstance.start;
                    const end = scrollTriggerInstance.end;
                    const targetScroll = start + (idx * 0.5 * (end - start));
                    window.scrollTo({
                        top: targetScroll,
                        behavior: "smooth"
                    });
                }
            });
        });

        return () => {
            // Cleanup on matches change
        };
    });

    // Simulated CLI Typing and status indicators inside active viewport panel
    initBlinkingPrompts(panels);
    initClaudeMascotAnimations();
}

/**
 * Animates the orange cyber-mascot in the Claude Panel using GSAP
 */
function initClaudeMascotAnimations() {
    if (typeof gsap === 'undefined') return;

    const mascot = document.querySelector('.claude-mascot-container');
    if (!mascot) return;

    const shadow = mascot.querySelector('.claude-mascot-shadow');
    const ascii = mascot.querySelector('.claude-ascii-mascot');

    // 1. Gentle vertical floating (idle)
    gsap.to(mascot, {
        y: -6,
        duration: 2,
        ease: "power1.inOut",
        yoyo: true,
        repeat: -1
    });

    // 2. Matching floating scale for the shadow
    if (shadow) {
        gsap.to(shadow, {
            scaleX: 0.8,
            opacity: 0.2,
            duration: 2,
            ease: "power1.inOut",
            yoyo: true,
            repeat: -1
        });
    }

    // 3. Ambient glow pulse for the ASCII mascot text glow shadow
    if (ascii) {
        gsap.to(ascii, {
            textShadow: "0 0 18px rgba(215, 119, 87, 0.8)",
            duration: 1.5,
            yoyo: true,
            repeat: -1,
            ease: "sine.inOut"
        });
    }
}

/**
 * Simulates subtle prompt activities in the CLI sessions
 */
function initBlinkingPrompts(panels) {
    // 1. Blinking cursor simulator in the footer inputs
    panels.forEach((panel, idx) => {
        const inputContainer = panel.querySelector('.cli-cursor-input');
        if (!inputContainer) return;

        const texts = ["", "help --agent", "status", "clear"];
        let currentTextIdx = 0;
        let charIdx = 0;
        let typing = true;

        function typeLoop() {
            const fullText = texts[currentTextIdx];
            if (typing) {
                inputContainer.textContent = fullText.substring(0, charIdx + 1);
                charIdx++;
                if (charIdx === fullText.length) {
                    typing = false;
                    setTimeout(typeLoop, 4000);
                } else {
                    setTimeout(typeLoop, 120 + Math.random() * 100);
                }
            } else {
                inputContainer.textContent = fullText.substring(0, charIdx - 1);
                charIdx--;
                if (charIdx <= 0) {
                    typing = true;
                    currentTextIdx = (currentTextIdx + 1) % texts.length;
                    setTimeout(typeLoop, 1000);
                } else {
                    setTimeout(typeLoop, 60);
                }
            }
        }

        setTimeout(typeLoop, idx * 1500 + 1000);
    });

    // 2. Main Sequential high-fidelity terminal script simulation using GSAP
    panels.forEach((panel) => {
        const targets = panel.querySelectorAll('.animation-target');
        const diffBlock = panel.querySelector('.cli-diff, .grok-write-diff');
        const typingBlock = panel.querySelector('.typing-container');
        const mascot = document.querySelector('.claude-mascot-container');

        if (targets.length === 0) return;

        // Custom individual timeline for each panel
        const timeline = gsap.timeline({
            repeat: -1,
            repeatDelay: 5,
            delay: Math.random() * 2 // stagger start slightly
        });

        // Hide targets initially
        gsap.set(targets, { opacity: 0, display: "none" });
        if (diffBlock) {
            gsap.set(diffBlock, { height: 0, overflow: "hidden" });
        }

        // Sequential step revealing
        targets.forEach((target) => {
            timeline.add(() => {
                target.style.display = "block";
                if (target.tagName === 'UL' || target.classList.contains('cli-todo-list')) {
                    target.style.display = "flex";
                }

                // Specific reactions when certain blocks reveal
                if (target.classList.contains('cli-command') || target.classList.contains('cli-command-codex')) {
                    // Trigger Mascot quick scale reaction when command is parsed
                    if (mascot) {
                        gsap.to(mascot, { scale: 1.15, duration: 0.2, yoyo: true, repeat: 1 });
                    }
                }
            });

            timeline.to(target, {
                opacity: 1,
                y: 0,
                duration: 0.5,
                ease: "power2.out"
            });

            // If it contains the typing container text
            if (typingBlock && target.contains(typingBlock)) {
                const finalStr = typingBlock.textContent;
                timeline.add(() => {
                    typingBlock.textContent = "";
                    let charI = 0;
                    function type() {
                        if (charI < finalStr.length) {
                            typingBlock.textContent += finalStr[charI];
                            charI++;
                            setTimeout(type, 30);
                        }
                    }
                    type();
                });
                timeline.delay(1.5); // Give time for simulated typing
            }

            // Diffs expansion
            if (diffBlock && target.contains(diffBlock)) {
                timeline.to(diffBlock, {
                    height: "auto",
                    duration: 0.6,
                    ease: "power3.inOut"
                });
            }

            // Small delay between commands and answers
            timeline.delay(0.8);
        });

        // Special second target group (outputs & pre formatting)
        const targets2 = panel.querySelectorAll('.animation-target2');
        if (targets2.length > 0) {
            timeline.add(() => {
                gsap.set(targets2, { display: "block" });
            });
            timeline.to(targets2, {
                opacity: 1,
                duration: 0.4,
                stagger: 0.2
            });
        }
    });
}
