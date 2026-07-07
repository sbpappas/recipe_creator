// front-end script that shows a loading overlay when recipe requests are in-flight

document.addEventListener('DOMContentLoaded', function () {
  function showLoading() {
    let overlay = document.getElementById('loading-overlay');
    if (!overlay) {
      overlay = document.createElement('div');
      overlay.id = 'loading-overlay';
      overlay.className = 'loading-overlay';
      overlay.innerHTML = `
        <div class="loading-box" role="status" aria-live="polite">
          <div class="spinner-large" aria-hidden="true"></div>
          <div class="loading-text">Thinking... this may take a minute</div>
        </div>`;
      document.body.appendChild(overlay);
    }
    overlay.classList.add('active');
  }

  function hideLoading() {
    const overlay = document.getElementById('loading-overlay');
    if (overlay) overlay.classList.remove('active');
  }

  function isRecipeGenerationForm(form) {
    if (!(form instanceof HTMLFormElement)) return false;
    const action = (form.getAttribute('action') || form.getAttribute('hx-post') || window.location.pathname).trim();
    return action === '/recipes/new' || action.endsWith('/recipes/new');
  }

  // Show loading for any internal recipe link navigation
  document.addEventListener('click', function (e) {
    const a = e.target.closest('a');
    if (!a) return;
    const href = a.getAttribute('href');
    if (!href) return;
    if (href.startsWith('/recipes/') && !href.includes('/new')) {
      showLoading();
      setTimeout(hideLoading, 15000);
    }
  });

  document.addEventListener('click', function (e) {
    const button = e.target.closest('button[type="submit"], input[type="submit"]');
    if (!button) return;
    const form = button.closest('form');
    if (isRecipeGenerationForm(form)) {
      showLoading();
    }
  }, true);

  document.addEventListener('submit', function (e) {
    const form = e.target;
    if (isRecipeGenerationForm(form)) {
      showLoading();
    }
  });

  document.addEventListener('htmx:beforeRequest', function (e) {
    const form = e.target;
    if (isRecipeGenerationForm(form)) {
      showLoading();
    }
  });

  document.addEventListener('htmx:afterRequest', function (e) {
    const form = e.target;
    if (isRecipeGenerationForm(form)) {
      hideLoading();
    }
  });

  document.addEventListener('htmx:responseError', function (e) {
    const form = e.target;
    if (isRecipeGenerationForm(form)) {
      hideLoading();
    }
  });

  window.addEventListener('load', hideLoading);
  window.addEventListener('pageshow', hideLoading);
});
