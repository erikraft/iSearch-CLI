/* ============================================================
   ISEARCH CLI™ — OFFICIAL DOWNLOAD CENTER INTERACTIVE CORE
   ============================================================ */

// Top-level Configurable Variables (Do not hardcode release version tags)
const CONFIG = {
    GITHUB_API_URL: "https://api.github.com/repos/erikraft/iSearch-CLI/releases/latest",
    FALLBACK_VERSION: "0.1.0", // Fallback version if offline or API limit reached
    REPO_URL: "https://github.com/erikraft/iSearch-CLI"
};

document.addEventListener("DOMContentLoaded", () => {
    // Initializations
    initLoader();
    initHeaderScroll();
    initMobileMenu();
    initThreeBg();
    initTyping();
    initScrollReveal();
    initCounters();
    initScrollProgress();
    initClipboard();
    initTabs();

    // Dynamic Releases Fetcher
    fetchLatestRelease();
});

/**
 * Site Loader Handler
 */
function initLoader() {
    const loader = document.querySelector('.site-loader');
    if (!loader) return;

    window.addEventListener('load', () => {
        setTimeout(() => {
            loader.classList.add('hidden');
            document.body.classList.remove('loading');

            // Trigger Hero entry animations via GSAP
            if (typeof gsap !== 'undefined') {
                gsap.from('.hero-content > *', {
                    opacity: 0,
                    y: 30,
                    duration: 1,
                    stagger: 0.12,
                    ease: "power3.out"
                });

                gsap.from('.hero-terminal-wrapper', {
                    opacity: 0,
                    y: 50,
                    duration: 1.4,
                    ease: "power3.out",
                    delay: 0.3
                });

                // Smooth continuous marquee text translation
                const marqueeInner = document.querySelector('.hero-marquee-inner');
                if (marqueeInner) {
                    const marqueeText = marqueeInner.textContent;
                    marqueeInner.textContent = marqueeText + ' ' + marqueeText;

                    gsap.to(marqueeInner, {
                        xPercent: -50,
                        ease: "none",
                        duration: 40,
                        repeat: -1
                    });
                }
            }
        }, 500);
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
 * Mobile Navigation Toggle
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

    // Close menu on navigation click or link click
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
    const particlesCount = window.innerWidth < 768 ? 100 : 200;
    const geometry = new THREE.BufferGeometry();
    const positions = new Float32Array(particlesCount * 3);
    const colors = new Float32Array(particlesCount * 3);

    const palette = [
        new THREE.Color('#0077ff'), // Blue
        new THREE.Color('#8b5cf6'), // Purple
        new THREE.Color('#06b6d4'), // Cyan
        new THREE.Color('#c5bba8')  // Beige
    ];

    for (let i = 0; i < particlesCount; i++) {
        positions[i * 3] = (Math.random() - 0.5) * 60;
        positions[i * 3 + 1] = (Math.random() - 0.5) * 60;
        positions[i * 3 + 2] = (Math.random() - 0.5) * 60;

        const chosenColor = palette[Math.floor(Math.random() * palette.length)];
        colors[i * 3] = chosenColor.r;
        colors[i * 3 + 1] = chosenColor.g;
        colors[i * 3 + 2] = chosenColor.b;
    }

    geometry.setAttribute('position', new THREE.BufferAttribute(positions, 3));
    geometry.setAttribute('color', new THREE.BufferAttribute(colors, 3));

    // Material with transparent round particle simulation
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
        size: 0.5,
        sizeAttenuation: true,
        vertexColors: true,
        transparent: true,
        opacity: 0.6,
        map: texture,
        depthWrite: false,
        blending: THREE.AdditiveBlending
    });

    const particles = new THREE.Points(geometry, material);
    scene.add(particles);

    // Mouse movement response
    let mouseX = 0, mouseY = 0, targetX = 0, targetY = 0;

    window.addEventListener('mousemove', (e) => {
        mouseX = (e.clientX - window.innerWidth / 2) / 150;
        mouseY = (e.clientY - window.innerHeight / 2) / 150;
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

        particles.rotation.y = elapsedTime * 0.015;
        particles.rotation.x = elapsedTime * 0.008;

        targetX += (mouseX - targetX) * 0.05;
        targetY += (mouseY - targetY) * 0.05;

        particles.position.x = targetX * 1.2;
        particles.position.y = -targetY * 1.2;

        renderer.render(scene, camera);
    }

    animate();
}

/**
 * Typing Animation in Hero Title
 */
function initTyping() {
    const target = document.querySelector('.typing-text');
    if (!target) return;

    const phrases = [
        'Professional Terminal Browser',
        'Built with Rust',
        'Cross-platform & Fast',
        'Open Source Project'
    ];

    let phraseIdx = 0;
    let charIdx = 0;
    let isDeleting = false;
    let typeSpeed = 80;

    function type() {
        const currentPhrase = phrases[phraseIdx];

        if (isDeleting) {
            target.textContent = currentPhrase.substring(0, charIdx - 1);
            charIdx--;
            typeSpeed = 40;
        } else {
            target.textContent = currentPhrase.substring(0, charIdx + 1);
            charIdx++;
            typeSpeed = 80;
        }

        if (!isDeleting && charIdx === currentPhrase.length) {
            isDeleting = true;
            typeSpeed = 2000; // Pause at end of typing
        } else if (isDeleting && charIdx === 0) {
            isDeleting = false;
            phraseIdx = (phraseIdx + 1) % phrases.length;
            typeSpeed = 400; // Pause before next phrase
        }

        setTimeout(type, typeSpeed);
    }

    type();

    // Start subtle typing loops in Hero terminal simulator mockup
    initTerminalMockupTyping();
}

/**
 * Terminal simulator state machine loop & animations
 */
function initTerminalMockupTyping() {
    const bootContainer = document.getElementById('cli-boot-loader');
    const bannerScreen = document.getElementById('cli-banner-screen');
    const outputScreen = document.getElementById('cli-output-screen');
    const cursorInput = document.querySelector('.cli-cursor-input');

    if (!bootContainer || !bannerScreen || !outputScreen || !cursorInput) return;

    const bootLines = Array.from(bootContainer.querySelectorAll('.boot-line'));
    let currentBootIdx = 0;

    // Phase 1: Boot sequence simulation
    function simulateBoot() {
        if (currentBootIdx < bootLines.length) {
            // Activate current line
            bootLines[currentBootIdx].classList.add('active');

            // Random natural loading delay (200ms - 600ms)
            const delay = 150 + Math.random() * 300;
            currentBootIdx++;
            setTimeout(simulateBoot, delay);
        } else {
            // Once boot is complete, wait 1s, then fade to active shell banner
            setTimeout(() => {
                bootContainer.style.display = 'none';
                bannerScreen.style.display = 'block';
                // Start typing loop
                startTypingLoop();
            }, 1000);
        }
    }

    // Start boot cycle after a small initial load
    setTimeout(simulateBoot, 500);

    // Phase 2: Natural Typing loop simulating real commands and views
    const commandsList = [
        { cmd: "help", response: "help" },
        { cmd: "browse https://github.com/erikraft/iSearch-CLI", response: "browse" },
        { cmd: "version --check", response: "version" },
        { cmd: "self-update", response: "update" },
        { cmd: "donate", response: "donate" }
    ];

    let cmdIdx = 0;
    let charI = 0;
    let deleting = false;

    function startTypingLoop() {
        function typeCmd() {
            const currentItem = commandsList[cmdIdx];
            const currentCmd = currentItem.cmd;

            if (!deleting) {
                cursorInput.textContent = currentCmd.substring(0, charI + 1);
                charI++;
                if (charI === currentCmd.length) {
                    // Pause on the finished command before action
                    setTimeout(() => {
                        triggerTerminalAction(currentItem.response);
                    }, 1200);
                } else {
                    setTimeout(typeCmd, 70 + Math.random() * 80); // Natural random typing
                }
            } else {
                cursorInput.textContent = currentCmd.substring(0, charI - 1);
                charI--;
                if (charI === 0) {
                    deleting = false;
                    cmdIdx = (cmdIdx + 1) % commandsList.length;
                    setTimeout(typeCmd, 600); // Pause before typing the next command
                } else {
                    setTimeout(typeCmd, 30); // Fast backspacing
                }
            }
        }

        typeCmd();

        function triggerTerminalAction(actionType) {
            if (actionType === "browse") {
                // Fade to TUI mode
                bannerScreen.style.display = 'none';
                outputScreen.style.display = 'block';

                // Remain in TUI Mode for 6 seconds, then restore and continue typing
                setTimeout(() => {
                    outputScreen.style.display = 'none';
                    bannerScreen.style.display = 'block';
                    deleting = true;
                    setTimeout(typeCmd, 400);
                }, 6000);
            } else {
                // For simple commands, just backspace directly after 2 seconds
                setTimeout(() => {
                    deleting = true;
                    typeCmd();
                }, 2000);
            }
        }
    }
}

/**
 * Scroll Reveal Animation
 */
function initScrollReveal() {
    const reveals = document.querySelectorAll('.reveal');

    const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.classList.add('is-visible');
            }
        });
    }, {
        threshold: 0.1,
        rootMargin: "0px 0px -40px 0px"
    });

    reveals.forEach(el => observer.observe(el));
}

/**
 * Numeric Count-Up animations
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
                    duration: 1.8,
                    ease: "power3.out",
                    onUpdate: () => {
                        target.textContent = Math.floor(obj.val);
                    },
                    onComplete: () => {
                        target.textContent = countTo + (target.getAttribute('data-target') === "100" ? "%" : "");
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
    bar.style.boxShadow = '0 0 8px var(--accent-light)';

    document.body.appendChild(bar);

    window.addEventListener('scroll', () => {
        const scrolled = (window.scrollY / (document.documentElement.scrollHeight - window.innerHeight)) * 100;
        bar.style.width = scrolled + '%';
    });
}

/**
 * Clipboard Copy Handler
 */
function initClipboard() {
    const copyBtns = document.querySelectorAll('.copy-btn');

    copyBtns.forEach(btn => {
        btn.addEventListener('click', () => {
            const textToCopy = btn.getAttribute('data-clipboard');
            if (!textToCopy) return;

            navigator.clipboard.writeText(textToCopy).then(() => {
                const originalHtml = btn.innerHTML;
                btn.innerHTML = '<i class="fa-solid fa-check"></i> Copied!';
                btn.classList.add('copied');

                setTimeout(() => {
                    btn.innerHTML = originalHtml;
                    btn.classList.remove('copied');
                }, 2000);
            }).catch(err => {
                console.error('Failed to copy text: ', err);
            });
        });
    });
}

/**
 * Interactive Tabs System
 */
function initTabs() {
    const triggers = document.querySelectorAll('.tab-trigger');
    const panels = document.querySelectorAll('.tab-panel');

    triggers.forEach(trigger => {
        trigger.addEventListener('click', () => {
            const targetId = trigger.getAttribute('aria-controls');

            // Deactivate all triggers & panels
            triggers.forEach(t => {
                t.classList.remove('active');
                t.setAttribute('aria-selected', 'false');
            });
            panels.forEach(p => p.classList.remove('active'));

            // Activate chosen trigger & panel
            trigger.classList.add('active');
            trigger.setAttribute('aria-selected', 'true');
            const targetPanel = document.getElementById(targetId);
            if (targetPanel) {
                targetPanel.classList.add('active');
            }
        });
    });
}

/**
 * Dynamic GitHub Releases Fetcher
 */
function fetchLatestRelease() {
    // Keep URLs and release versions fully configurable dynamically
    const versionTags = document.querySelectorAll('.release-version-tag');
    const downloadBtns = document.querySelectorAll('.download-link-btn');
    const sourceZipBtn = document.querySelector('.source-zip-btn');
    const sourceTarBtn = document.querySelector('.source-tar-btn');
    const releaseNotesLinks = document.querySelectorAll('.release-notes-link');

    // Default static update based on CONFIG
    updateReleaseDOM(CONFIG.FALLBACK_VERSION);

    fetch(CONFIG.GITHUB_API_URL)
        .then(response => {
            if (!response.ok) {
                throw new Error("API Limit reached or network error.");
            }
            return response.json();
        })
        .then(data => {
            const tagName = data.tag_name;
            const cleanVersion = tagName.replace(/^v/, ''); // Remove 'v' prefix if exists

            // Update DOM with dynamic latest release info
            updateReleaseDOM(cleanVersion);

            // Populate specific dynamic assets if present in GitHub release assets structure
            if (data.assets && data.assets.length > 0) {
                downloadBtns.forEach(btn => {
                    const assetNamePattern = btn.getAttribute('data-asset');
                    if (assetNamePattern) {
                        // Find matching asset in API assets list
                        const matchedAsset = data.assets.find(asset => {
                            // Check if asset name matches the pattern (e.g. replacing v0.1.0 with the actual dynamic cleanVersion/tagName)
                            const parameterizedName = assetNamePattern.replace('v0.1.0', tagName);
                            return asset.name === parameterizedName || asset.name === assetNamePattern;
                        });

                        if (matchedAsset) {
                            btn.setAttribute('href', matchedAsset.browser_download_url);
                            // Set file size if present
                            const sizeInMb = (matchedAsset.size / (1024 * 1024)).toFixed(2);
                            const originalText = btn.innerHTML;
                            if (!originalText.includes('MB')) {
                                btn.innerHTML = `${originalText} <span style="font-size: 10px; opacity: 0.65;">(${sizeInMb} MB)</span>`;
                            }
                        }
                    }
                });
            }

            // Update Release Notes Links
            releaseNotesLinks.forEach(link => {
                link.setAttribute('href', data.html_url);
            });
        })
        .catch(err => {
            console.log("GitHub API request failed, using configured default values.", err);
        });

    function updateReleaseDOM(version) {
        // Update tags
        versionTags.forEach(el => {
            el.textContent = version;
        });

        // Update standard download buttons URLs with clean versions
        downloadBtns.forEach(btn => {
            const currentHref = btn.getAttribute('href');
            if (currentHref) {
                const updatedHref = currentHref.replace(/v\d+\.\d+\.\d+/, `v${version}`);
                btn.setAttribute('href', updatedHref);
            }
        });

        // Update Source Code Links
        if (sourceZipBtn) {
            sourceZipBtn.setAttribute('href', `${CONFIG.REPO_URL}/archive/refs/tags/v${version}.zip`);
        }
        if (sourceTarBtn) {
            sourceTarBtn.setAttribute('href', `${CONFIG.REPO_URL}/archive/refs/tags/v${version}.tar.gz`);
        }
    }
}
