(() => {
  const stylesheet = document.createElement('link');
  stylesheet.rel = 'stylesheet';
  stylesheet.href = './assets/styles/tailwind.generated.css';
  stylesheet.dataset.madrasaStaticTailwind = 'true';
  document.head.appendChild(stylesheet);
})();
