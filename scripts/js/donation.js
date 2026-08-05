/**
 * donation.js
 * Dynamic interactive logic for donation.html:
 * - Bilingual translation toggling & localStorage persistence.
 * - Current year dynamic replacement.
 * - UI animations & enhancements.
 */

document.addEventListener("DOMContentLoaded", () => {
  initLanguageToggle();
  initCurrentYear();
  initAnimations();
});

// ============================================================
// 1. Language Toggling System (Bilingual PT/EN)
// ============================================================

function initLanguageToggle() {
  const langToggleBtn = document.getElementById("lang-toggle-btn");
  if (!langToggleBtn) return;

  // Set default language based on localStorage or browser lang, default to PT
  let currentLang = localStorage.getItem("preferred-lang");
  if (!currentLang) {
    const browserLang = navigator.language || navigator.userLanguage;
    currentLang = browserLang.startsWith("pt") ? "pt-BR" : "en";
  }

  // Set initial translation
  updateLangUI(currentLang);

  // Toggle button click listener
  langToggleBtn.addEventListener("click", () => {
    const nextLang = document.documentElement.lang === "en" ? "pt-BR" : "en";
    updateLangUI(nextLang);
  });
}

function updateLangUI(lang) {
  document.documentElement.lang = lang;
  localStorage.setItem("preferred-lang", lang);

  const langToggleBtn = document.getElementById("lang-toggle-btn");
  const customInput = document.getElementById("custom-pix-value");

  // Adjust display of all .lang-pt and .lang-en elements
  const ptElements = document.querySelectorAll(".lang-pt");
  const enElements = document.querySelectorAll(".lang-en");

  if (lang === "en") {
    if (langToggleBtn) {
      langToggleBtn.innerHTML = '<i class="fa-solid fa-globe"></i> <span>PT</span>';
    }
    if (customInput) {
      customInput.placeholder = "Enter amount...";
    }
    ptElements.forEach(el => {
      el.style.display = "none";
    });
    enElements.forEach(el => {
      // Check if tag is h1, h2, h3, title, or p for semantic block/inline layout
      const tag = el.tagName.toLowerCase();
      if (tag === "span" || tag === "a" || tag === "i") {
        el.style.display = "inline";
      } else {
        el.style.display = "block";
      }
    });
  } else {
    if (langToggleBtn) {
      langToggleBtn.innerHTML = '<i class="fa-solid fa-globe"></i> <span>EN</span>';
    }
    if (customInput) {
      customInput.placeholder = "Digite o valor...";
    }
    enElements.forEach(el => {
      el.style.display = "none";
    });
    ptElements.forEach(el => {
      const tag = el.tagName.toLowerCase();
      if (tag === "span" || tag === "a" || tag === "i") {
        el.style.display = "inline";
      } else {
        el.style.display = "block";
      }
    });
  }
}

// ============================================================
// 2. Current Year Auto Update
// ============================================================

function initCurrentYear() {
  const years = document.querySelectorAll(".currentYearVal");
  const currentYear = new Date().getFullYear();
  years.forEach(year => {
    year.textContent = currentYear;
  });
}

// ============================================================
// 3. Page Layout Entry & Glow Animations (GSAP)
// ============================================================

function initAnimations() {
  const gsapInstance = window.gsap;
  if (!gsapInstance) return;

  // Stagger reveal cards
  gsapInstance.from(".support-card", {
    autoAlpha: 0,
    y: 40,
    duration: 0.9,
    stagger: 0.15,
    ease: "power3.out"
  });
}
