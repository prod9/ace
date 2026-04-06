(() => {
  const rail = document.querySelector(".matrix-rail");
  if (!rail) return;

  const glyphs = [".", "·", "░", "▀", "█"];
  const rows = [];
  let tickCount = 0;
  let cursorRow = 0;
  let cursorColumn = 0;
  let stepRow = 1;
  let stepColumn = 1;
  let lastFrameTime = 0;
  let nextStepDelay = 0;

  function randomGlyph() {
    return glyphs[Math.floor(Math.random() * glyphs.length)];
  }

  function easeOutSpring(t) {
    const clamped = Math.max(0, Math.min(1, t));
    return 1 - Math.cos(clamped * Math.PI * 4.5) * Math.exp(-clamped * 6);
  }

  function scheduleNextStep() {
    const cycle = (Math.sin(tickCount * 0.12) + 1) * 0.5;
    const eased = easeOutSpring(cycle);
    nextStepDelay = 220 + (1 - eased) * 900;
  }

  function colorMixForCell(rowIndex, columnIndex) {
    const phase = tickCount * 0.18 + rowIndex * 0.55 + columnIndex * 0.08;
    const mix = (Math.sin(phase) + Math.sin(phase * 0.42)) * 0.25 + 0.5;
    return mix.toFixed(3);
  }

  function advanceCursor(rowCount, columnCount) {
    if (Math.random() < 0.08) {
      stepRow = Math.random() < 0.5 ? -1 : 1;
    }

    if (Math.random() < 0.08) {
      stepColumn = Math.random() < 0.5 ? -1 : 1;
    }

    cursorRow += stepRow;
    cursorColumn += stepColumn;

    if (cursorRow < 0 || cursorRow >= rowCount) {
      stepRow *= -1;
      cursorRow += stepRow * 2;
    }

    if (cursorColumn < 0 || cursorColumn >= columnCount) {
      stepColumn *= -1;
      cursorColumn += stepColumn * 2;
    }
  }

  function mutateNextCell() {
    if (rows.length === 0) return;

    const rowIndex = cursorRow;
    const row = rows[rowIndex];
    const cells = row.children;
    if (cells.length === 0) return;

    const columnIndex = cursorColumn;
    const cell = cells[columnIndex];
    cell.textContent = randomGlyph();
    cell.style.setProperty("--matrix-mix", colorMixForCell(rowIndex, columnIndex));
    advanceCursor(rows.length, cells.length);
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
    const railWidth = rail.getBoundingClientRect().width;
    const charWidth = fontSize * 0.6;
    const columnCount = Math.max(1, Math.ceil(railWidth / charWidth));
    const rowCount = Math.max(1, Math.ceil(pageHeight / (fontSize * lineHeight)));

    rail.replaceChildren();
    rows.length = 0;
    cursorRow = 0;
    cursorColumn = 0;
    stepRow = 1;
    stepColumn = 1;
    lastFrameTime = 0;
    scheduleNextStep();

    for (let rowIndex = 0; rowIndex < rowCount; rowIndex += 1) {
      const row = document.createElement("div");
      row.className = "matrix-row";

      for (let columnIndex = 0; columnIndex < columnCount; columnIndex += 1) {
        const cell = document.createElement("span");
        cell.className = "matrix-cell";
        cell.textContent = randomGlyph();
        cell.style.setProperty("--matrix-mix", colorMixForCell(rowIndex, columnIndex));
        row.append(cell);
      }

      rows.push(row);
      rail.append(row);
    }
  }

  function tick() {
    tickCount += 1;
    mutateNextCell();
    scheduleNextStep();
  }

  let resizeTimer = 0;

  function handleResize() {
    window.clearTimeout(resizeTimer);
    resizeTimer = window.setTimeout(buildRail, 120);
  }

  function animate(frameTime) {
    if (lastFrameTime === 0) {
      lastFrameTime = frameTime;
    }

    const elapsed = frameTime - lastFrameTime;
    if (elapsed >= nextStepDelay) {
      tick();
      lastFrameTime = frameTime;
    }

    window.requestAnimationFrame(animate);
  }

  buildRail();
  window.addEventListener("resize", handleResize);
  window.addEventListener("load", buildRail, { once: true });
  window.requestAnimationFrame(animate);
})();
