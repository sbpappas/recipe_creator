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

  // Show loading for any internal recipe link navigation
  document.addEventListener('click', function (e) {
    const a = e.target.closest('a');
    if (!a) return;
    const href = a.getAttribute('href');
    if (!href) return;
    // Only intercept internal recipe show links
    if (href.startsWith('/recipes/') && !href.includes('/new')) {
      showLoading();
      // Let the navigation proceed; hide after a short timeout in case navigation fails
      setTimeout(hideLoading, 15000);
    }
  });

  // Show the overlay for recipe generation submissions, whether they use htmx or a normal form post
  document.addEventListener('submit', function (e) {
    const form = e.target;
    if (!(form instanceof HTMLFormElement)) return;
    const action = form.getAttribute('action') || window.location.pathname;
    if (action === '/recipes/new') {
      showLoading();
      setTimeout(hideLoading, 15000);
    }
  });

  // Hide overlay when page is shown (e.g., after navigation)
  window.addEventListener('pageshow', hideLoading);
});
