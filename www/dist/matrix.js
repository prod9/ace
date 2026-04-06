(() => {
  const rail = document.querySelector(".matrix-rail");
  if (!rail) return;

  const glyphs = [".", "·", "░", "▀", "█"];
  const columns = [];
  let tickCount = 0;

  function randomGlyph() {
    return glyphs[Math.floor(Math.random() * glyphs.length)];
  }

  function randomizeColumn(column, columnIndex) {
    const cells = column.children;

    for (const [rowIndex, cell] of Array.from(cells).entries()) {
      if (Math.random() >= 0.72) {
        cell.textContent = randomGlyph();
      }

      const phase = tickCount * 0.18 + rowIndex * 0.55 + columnIndex * 0.08;
      const mix = (Math.sin(phase) + Math.sin(phase * 0.42)) * 0.25 + 0.5;
      cell.style.setProperty("--matrix-mix", mix.toFixed(3));
    }
  }

  function buildRail() {
    const styles = getComputedStyle(document.documentElement);
    const fontSize = parseFloat(styles.getPropertyValue("--matrix-font-size")) || 24;
    const lineHeight = parseFloat(styles.getPropertyValue("--matrix-line-height")) || 1;
    const pageHeight = Math.max(
      document.documentElement.scrollHeight,
      document.body.scrollHeight,
      window.innerHeight,
    );
    const railWidth = rail.clientWidth;
    const columnCount = Math.max(1, Math.floor(railWidth / (fontSize * 0.95)));
    const rowCount = Math.max(1, Math.ceil(pageHeight / (fontSize * lineHeight)));

    rail.replaceChildren();
    columns.length = 0;

    for (let columnIndex = 0; columnIndex < columnCount; columnIndex += 1) {
      const column = document.createElement("pre");
      column.className = "matrix-column";

      for (let rowIndex = 0; rowIndex < rowCount; rowIndex += 1) {
        const cell = document.createElement("span");
        cell.className = "matrix-cell";
        cell.textContent = randomGlyph();
        column.append(cell);
      }

      columns.push(column);
      rail.append(column);
    }
  }

  function tick() {
    tickCount += 1;

    for (const [index, column] of columns.entries()) {
      randomizeColumn(column, index);
    }
  }

  let resizeTimer = 0;

  function handleResize() {
    window.clearTimeout(resizeTimer);
    resizeTimer = window.setTimeout(buildRail, 120);
  }

  buildRail();
  window.addEventListener("resize", handleResize);
  window.addEventListener("load", buildRail, { once: true });
  window.setInterval(tick, 60);
})();
