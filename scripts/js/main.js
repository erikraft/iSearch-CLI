/* ============================================================
   ISEARCH CLI™ — OFFICIAL DOWNLOAD CENTER INTERACTIVE CORE
   ============================================================ */

// Top-level Configurable Variables (Do not hardcode release version tags)
const CONFIG = {
    GITHUB_API_URL: "https://api.github.com/repos/erikraft/iSearch-CLI/releases/latest",
    REPO_URL: "https://github.com/erikraft/iSearch-CLI"
};

const DOWNLOAD_ASSETS = {
    windows: [
        { name: "isearch-installer-x86_64.exe", label: "isearch-installer-x86_64.exe", style: "button-primary", icon: "fa-solid fa-download" },
        { name: "isearch-installer-x86_64.msi", label: "isearch-installer-x86_64.msi", style: "button-secondary", icon: "fa-solid fa-box" },
        { name: "isearch-windows-x86_64.exe", label: "isearch-windows-x86_64.exe", style: "button-secondary", icon: "fa-solid fa-file" },
        { name: "isearch-windows-arm64.exe", label: "isearch-windows-arm64.exe", style: "button-secondary", icon: "fa-solid fa-file" }
    ],
    linux: [
        { name: "isearch-linux-x86_64", label: "isearch-linux-x86_64", style: "button-primary", icon: "fa-solid fa-download" },
        { name: "isearch-linux-aarch64", label: "isearch-linux-aarch64", style: "button-secondary", icon: "fa-solid fa-file" },
        { name: "isearch-linux-arm", label: "isearch-linux-arm", style: "button-secondary", icon: "fa-solid fa-file" },
        { name: "isearch-linux-x86_64.AppImage", label: "isearch-linux-x86_64.AppImage", style: "button-secondary", icon: "fa-solid fa-rocket" },
        { name: "isearch-linux-x86_64.deb", label: "isearch-linux-x86_64.deb", style: "button-secondary", icon: "fa-solid fa-file-archive" },
        { name: "isearch-linux-x86_64.rpm", label: "isearch-linux-x86_64.rpm", style: "button-secondary", icon: "fa-solid fa-file-archive" },
        { name: "isearch-linux-x86_64.tar.gz", label: "isearch-linux-x86_64.tar.gz", style: "button-secondary", icon: "fa-solid fa-file-archive" },
        { name: "isearch-linux-aarch64.tar.gz", label: "isearch-linux-aarch64.tar.gz", style: "button-secondary", icon: "fa-solid fa-file-archive" },
        { name: "isearch-linux-arm.tar.gz", label: "isearch-linux-arm.tar.gz", style: "button-secondary", icon: "fa-solid fa-file-archive" }
    ],
    macos: [
        { name: "isearch-installer-macos-aarch64.dmg", label: "isearch-installer-macos-aarch64.dmg", style: "button-primary", icon: "fa-solid fa-download" },
        { name: "isearch-installer-macos-aarch64.pkg", label: "isearch-installer-macos-aarch64.pkg", style: "button-secondary", icon: "fa-solid fa-box" },
        { name: "isearch-installer-macos-x86_64.dmg", label: "isearch-installer-macos-x86_64.dmg", style: "button-secondary", icon: "fa-solid fa-download" },
        { name: "isearch-installer-macos-x86_64.pkg", label: "isearch-installer-macos-x86_64.pkg", style: "button-secondary", icon: "fa-solid fa-box" },
        { name: "isearch-macos-aarch64", label: "isearch-macos-aarch64", style: "button-secondary", icon: "fa-solid fa-file" },
        { name: "isearch-macos-aarch64.tar.gz", label: "isearch-macos-aarch64.tar.gz", style: "button-secondary", icon: "fa-solid fa-file-archive" },
        { name: "isearch-macos-x86_64", label: "isearch-macos-x86_64", style: "button-secondary", icon: "fa-solid fa-file" },
        { name: "isearch-macos-x86_64.tar.gz", label: "isearch-macos-x86_64.tar.gz", style: "button-secondary", icon: "fa-solid fa-file-archive" }
    ],
    termux: [
        { name: "isearch-cli-termux-aarch64.tar.gz", label: "isearch-cli-termux-aarch64.tar.gz", style: "button-primary", icon: "fa-solid fa-download" },
        { name: "isearch-cli-termux-arm.tar.gz", label: "isearch-cli-termux-arm.tar.gz", style: "button-secondary", icon: "fa-solid fa-file-archive" },
        { name: "isearch-cli-termux-x64.tar.gz", label: "isearch-cli-termux-x64.tar.gz", style: "button-secondary", icon: "fa-solid fa-file-archive" }
    ],
    source: [
        { type: "zip", label: "Source ZIP (.zip)", style: "button-primary", icon: "fa-solid fa-file-zipper" },
        { type: "tar", label: "Source Tarball (.tar.gz)", style: "button-secondary", icon: "fa-solid fa-file-archive" }
    ]
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

    // Build download buttons and resolve latest release information
    initDownloadButtons();
    fetchLatestRelease();
});

/**
 * Helper to update progress bar characters and percentages
 */
function updateProgress(percent) {
    const totalBlocks = 20;
    const filledBlocks = Math.round((percent / 100) * totalBlocks);
    const emptyBlocks = totalBlocks - filledBlocks;
    const barStr = "[" + "█".repeat(filledBlocks) + "░".repeat(emptyBlocks) + "]";

    const barEl = document.querySelector('.progress-bar');
    const percentEl = document.querySelector('.progress-percent');
    if (barEl) barEl.textContent = barStr;
    if (percentEl) percentEl.textContent = percent + "%";
}

/**
 * Site Loader Handler
 */
function initLoader() {
    const loader = document.querySelector('.site-loader');
    if (!loader) return;

    // Check if user prefers reduced motion
    const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;

    if (prefersReducedMotion) {
        // Fast-track initial state if motion is reduced
        loader.classList.add('hidden');
        document.body.classList.remove('loading');
        triggerEntranceAnimations();
        return;
    }

    // Command to type
    const commandText = "init --isearch-download";
    const cmdTextEl = document.querySelector('.cmd-text');
    const loaderCursor = document.querySelector('.loader-cursor');

    // Create a GSAP Timeline for the terminal booting sequence
    const tl = gsap.timeline({
        onComplete: () => {
            // Once boot is completely finished, fade out loader
            gsap.to(loader, {
                opacity: 0,
                duration: 0.5,
                ease: "power2.out",
                onComplete: () => {
                    loader.classList.add('hidden');
                    document.body.classList.remove('loading');
                    triggerEntranceAnimations();
                }
            });
        }
    });

    // 1. Type out the command line
    let typedObj = { length: 0 };
    tl.to(typedObj, {
        length: commandText.length,
        duration: 0.8,
        ease: "none",
        onUpdate: () => {
            const currentLen = Math.floor(typedObj.length);
            if (cmdTextEl) cmdTextEl.textContent = commandText.substring(0, currentLen);
        }
    });

    // 2. Pause slightly with command typed, then hide first cursor
    tl.to({}, { duration: 0.25 });
    tl.call(() => {
        if (loaderCursor) loaderCursor.style.display = 'none';
    });

    // 3. Sequentially show [ OK ] log lines
    const logLines = document.querySelectorAll('.log-line');
    logLines.forEach((line) => {
        tl.call(() => {
            line.style.display = 'flex';
        });
        tl.to(line, {
            opacity: 1,
            duration: 0.15,
            ease: "power1.out"
        });
        tl.to({}, { duration: 0.1 });
    });

    // 4. Show the progress bar line
    const progressLine = document.querySelector('.progress-line');
    tl.call(() => {
        if (progressLine) progressLine.style.display = 'flex';
    });
    tl.to(progressLine, {
        opacity: 1,
        duration: 0.15,
        ease: "power1.out"
    });

    // 5. Progressively update loading bar
    let progressObj = { val: 0 };
    tl.to(progressObj, {
        val: 100,
        duration: 1.2,
        ease: "power1.inOut",
        onUpdate: () => {
            updateProgress(Math.floor(progressObj.val));
        }
    });

    // 6. Show final prompt line
    const finalLine = document.querySelector('.final-line');
    tl.call(() => {
        if (finalLine) finalLine.style.display = 'flex';
    });
    tl.to(finalLine, {
        opacity: 1,
        duration: 0.15,
        ease: "power1.out"
    });

    // 7. Pause briefly before completing the boot
    tl.to({}, { duration: 0.4 });
}

function triggerEntranceAnimations() {
    // Trigger Hero entry animations via GSAP
    if (typeof gsap !== 'undefined') {
        gsap.from('.hero-content > *', {
            opacity: 0,
            y: 30,
            duration: 1,
            stagger: 0.12,
            ease: "power3.out"
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
 * Build download buttons dynamically on the page.
 */
function initDownloadButtons() {
    const assetSections = {
        windows: document.getElementById('windows-downloads'),
        linux: document.getElementById('linux-downloads'),
        macos: document.getElementById('macos-downloads'),
        termux: document.getElementById('termux-downloads'),
        source: document.getElementById('source-downloads')
    };

    Object.keys(DOWNLOAD_ASSETS).forEach(sectionKey => {
        const sectionElement = assetSections[sectionKey];
        if (!sectionElement) return;

        DOWNLOAD_ASSETS[sectionKey].forEach(asset => {
            const button = document.createElement('a');
            const isSource = sectionKey === 'source';
            const assetName = isSource ? asset.type : asset.name;
            const href = isSource
                ? `${CONFIG.REPO_URL}/releases/latest`
                : `https://github.com/erikraft/iSearch-CLI/releases/latest/download/${assetName}`;

            button.className = `button ${asset.style} ${isSource ? (asset.type === 'zip' ? 'source-zip-btn' : 'source-tar-btn') : 'download-link-btn'}`;
            button.setAttribute('href', href);
            if (!isSource) {
                button.setAttribute('data-asset', assetName);
                button.setAttribute('target', '_blank');
                button.setAttribute('rel', 'noopener noreferrer');
            } else {
                button.setAttribute('target', '_blank');
                button.setAttribute('rel', 'noopener noreferrer');
            }

            button.innerHTML = `<i class="${asset.icon}"></i> ${asset.label}`;
            sectionElement.appendChild(button);
        });
    });
}

/**
 * Dynamic GitHub Releases Fetcher
 */
function fetchLatestRelease() {
    const versionTags = document.querySelectorAll('.release-version-tag');
    const downloadBtns = document.querySelectorAll('.download-link-btn');
    const sourceZipBtn = document.querySelector('.source-zip-btn');
    const sourceTarBtn = document.querySelector('.source-tar-btn');
    const releaseNotesLinks = document.querySelectorAll('.release-notes-link');

    // Update tags immediately to a generic label until release data arrives
    versionTags.forEach(el => el.textContent = 'latest');

    fetch(CONFIG.GITHUB_API_URL)
        .then(response => {
            if (!response.ok) {
                throw new Error('API Limit reached or network error.');
            }
            return response.json();
        })
        .then(data => {
            const tagName = data.tag_name || 'latest';
            const cleanVersion = tagName.replace(/^v/, '');

            versionTags.forEach(el => el.textContent = cleanVersion);

            if (data.assets && data.assets.length > 0) {
                downloadBtns.forEach(btn => {
                    const assetName = btn.getAttribute('data-asset');
                    if (!assetName) return;

                    const matchedAsset = data.assets.find(asset => asset.name === assetName);
                    if (matchedAsset) {
                        btn.setAttribute('href', matchedAsset.browser_download_url);
                        const sizeInMb = (matchedAsset.size / (1024 * 1024)).toFixed(2);
                        const originalText = btn.innerHTML;
                        if (!originalText.includes('MB')) {
                            btn.innerHTML = `${originalText} <span style="font-size: 10px; opacity: 0.65;">(${sizeInMb} MB)</span>`;
                        }
                    }
                });
            }

            if (sourceZipBtn) {
                sourceZipBtn.setAttribute('href', `${CONFIG.REPO_URL}/archive/refs/tags/${tagName}.zip`);
            }
            if (sourceTarBtn) {
                sourceTarBtn.setAttribute('href', `${CONFIG.REPO_URL}/archive/refs/tags/${tagName}.tar.gz`);
            }

            releaseNotesLinks.forEach(link => {
                link.setAttribute('href', data.html_url || `${CONFIG.REPO_URL}/releases/latest`);
            });
        })
        .catch(err => {
            console.log('GitHub API request failed, using fallback latest URLs.', err);
            releaseNotesLinks.forEach(link => link.setAttribute('href', `${CONFIG.REPO_URL}/releases/latest`));
        });
}
