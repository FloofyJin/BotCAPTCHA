// Global state
let currentChallenge = null;
let timerInterval = null;
let selectedGridCells = []; // [{col, row}, ...] in click order

// DOM elements
const startBtn     = document.getElementById('startBtn');
const submitBtn    = document.getElementById('submitBtn');
const statusDiv    = document.getElementById('status');
const textDisplay  = document.getElementById('challenge-text');
const answerInput  = document.getElementById('answer-input');
const coordList    = document.getElementById('coord-list');
const gridContainer = document.getElementById('grid-container');

// ── Grid rendering ────────────────────────────────────────────────────────────

function renderGrid(gridSize) {
    gridContainer.innerHTML = '';
    gridContainer.style.gridTemplateColumns = `repeat(${gridSize}, 38px)`;

    for (let row = 0; row < gridSize; row++) {
        for (let col = 0; col < gridSize; col++) {
            const cell = document.createElement('div');
            cell.className = 'grid-cell';
            cell.dataset.col = col;
            cell.dataset.row = row;
            cell.addEventListener('click', () => handleCellClick(col, row));
            gridContainer.appendChild(cell);
        }
    }
}

function handleCellClick(col, row) {
    const existing = selectedGridCells.findIndex(c => c.col === col && c.row === row);
    if (existing !== -1) {
        selectedGridCells.splice(existing, 1); // deselect
    } else {
        selectedGridCells.push({ col, row });
    }
    updateGridDisplay();
}

function updateGridDisplay() {
    // Clear all order numbers and selection highlights
    document.querySelectorAll('.grid-cell').forEach(cell => {
        cell.textContent = '';
        cell.classList.remove('selected');
    });

    // Re-draw numbers for selected cells
    selectedGridCells.forEach((coord, idx) => {
        const cell = document.querySelector(
            `.grid-cell[data-col="${coord.col}"][data-row="${coord.row}"]`
        );
        if (cell) {
            cell.textContent = idx + 1;
            cell.classList.add('selected');
        }
    });
}

function showCoordText(text) {
    coordList.textContent = text;
}

function resetGrid() {
    selectedGridCells = [];
    gridContainer.innerHTML = '';
    coordList.innerHTML = '<span class="coord-placeholder">[ Target squares will appear here in jumbled order ]</span>';
}

// ── Timer ─────────────────────────────────────────────────────────────────────

function updateTimerStatus(remainingMs) {
    statusDiv.innerHTML = `Time remaining: <span class="timer">${(remainingMs / 1000).toFixed(1)}s</span>`;
}

function stopTimer() {
    if (timerInterval) { clearInterval(timerInterval); timerInterval = null; }
}

// ── Challenge lifecycle ───────────────────────────────────────────────────────

async function startChallenge() {
    stopTimer();
    currentChallenge = null;
    selectedGridCells = [];

    textDisplay.textContent = 'Loading...';
    textDisplay.classList.add('placeholder');
    answerInput.value = '';
    resetGrid();

    startBtn.disabled = true;
    submitBtn.disabled = true;
    statusDiv.textContent = 'Fetching challenge...';

    try {
        const response = await fetch('/api/challenge');
        currentChallenge = await response.json();
        window.currentChallenge = currentChallenge;

        // Render word puzzle
        textDisplay.textContent = currentChallenge.text_content;
        textDisplay.classList.remove('placeholder');

        // Render grid puzzle
        renderGrid(currentChallenge.grid_size);
        showCoordText(currentChallenge.grid_coords_text);

        submitBtn.disabled = false;

        // Countdown timer
        let remaining = currentChallenge.duration_ms;
        updateTimerStatus(remaining);
        timerInterval = setInterval(() => {
            remaining -= 100;
            if (remaining <= 0) {
                stopTimer();
                submitBtn.disabled = true;
                startBtn.disabled = false;
                statusDiv.innerHTML = '<span class="error">Time expired! Click "Start Challenge" to try again.</span>';
            } else {
                updateTimerStatus(remaining);
            }
        }, 100);

    } catch (error) {
        statusDiv.innerHTML = `<span class="error">Error: ${error.message}</span>`;
        textDisplay.textContent = '[ Error loading challenge ]';
        textDisplay.classList.add('placeholder');
        startBtn.disabled = false;
    }
}

// ── Submit ────────────────────────────────────────────────────────────────────

async function submitAnswer() {
    if (!currentChallenge) {
        statusDiv.innerHTML = '<span class="error">No active challenge.</span>';
        return;
    }

    // Parse word frequency JSON
    let wordAnswer;
    try {
        wordAnswer = JSON.parse(answerInput.value);
    } catch (e) {
        statusDiv.innerHTML = '<span class="error">Invalid JSON in word frequency field — check formatting.</span>';
        return;
    }
    if (typeof wordAnswer !== 'object' || Array.isArray(wordAnswer)) {
        statusDiv.innerHTML = '<span class="error">Word answer must be a JSON object: {"word": count, ...}</span>';
        return;
    }

    submitBtn.disabled = true;
    stopTimer();
    statusDiv.textContent = 'Submitting...';

    try {
        const response = await fetch('/api/submit', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                challenge_id: currentChallenge.challenge_id,
                answer: wordAnswer,
                grid_answer: selectedGridCells,
            }),
        });

        const result = await response.json();

        if (result.success) {
            statusDiv.innerHTML = `<span class="success">✓ ${result.message} — Click "Start Challenge" to try again.</span>`;
        } else {
            statusDiv.innerHTML = `<span class="error">✗ ${result.message} — Click "Start Challenge" to try again.</span>`;
        }

    } catch (error) {
        statusDiv.innerHTML = `<span class="error">Error: ${error.message}</span>`;
    }

    currentChallenge = null;
    selectedGridCells = [];
    startBtn.disabled = false;
    submitBtn.disabled = true;
}

// ── Event listeners ───────────────────────────────────────────────────────────

startBtn.addEventListener('click', startChallenge);
submitBtn.addEventListener('click', submitAnswer);

// ── Programmatic API for AI integration ──────────────────────────────────────

window.BotCaptchaAPI = {
    // Returns { challenge_id, text_content, duration_ms, grid_size, grid_coords_text }
    // grid_coords_text is a natural-language paragraph describing target cells in scrambled order
    async getChallenge() {
        const response = await fetch('/api/challenge');
        return await response.json();
    },

    getCurrentChallenge() {
        return currentChallenge;
    },

    // wordFrequencies: { word: count, ... }
    // sortedCoords:    [{col, row}, ...] in reading order (row asc, col asc)
    async submitChallenge(challengeId, wordFrequencies, sortedCoords) {
        const response = await fetch('/api/submit', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                challenge_id: challengeId,
                answer: wordFrequencies,
                grid_answer: sortedCoords,
            }),
        });
        return await response.json();
    },
};
