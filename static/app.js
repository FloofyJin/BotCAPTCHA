// Global state
let currentChallenge = null;
let timerInterval = null;

// DOM elements
const startBtn = document.getElementById('startBtn');
const submitBtn = document.getElementById('submitBtn');
const statusDiv = document.getElementById('status');
const textDisplay = document.getElementById('challenge-text');
const answerInput = document.getElementById('answer-input');

// Start a new challenge
async function startChallenge() {
    stopTimer();
    currentChallenge = null;
    textDisplay.textContent = 'Loading...';
    textDisplay.classList.add('placeholder');
    answerInput.value = '';

    startBtn.disabled = true;
    submitBtn.disabled = true;
    statusDiv.textContent = 'Fetching challenge...';

    try {
        const response = await fetch('/api/challenge');
        currentChallenge = await response.json();
        window.currentChallenge = currentChallenge;

        // Display the passage
        textDisplay.textContent = currentChallenge.text_content;
        textDisplay.classList.remove('placeholder');

        submitBtn.disabled = false;

        // Start countdown
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

function updateTimerStatus(remainingMs) {
    const seconds = (remainingMs / 1000).toFixed(1);
    statusDiv.innerHTML = `Time remaining: <span class="timer">${seconds}s</span>`;
}

function stopTimer() {
    if (timerInterval) {
        clearInterval(timerInterval);
        timerInterval = null;
    }
}

// Submit the word frequency answer
async function submitAnswer() {
    if (!currentChallenge) {
        statusDiv.innerHTML = '<span class="error">No active challenge.</span>';
        return;
    }

    // Parse the JSON answer
    let answer;
    try {
        answer = JSON.parse(answerInput.value);
    } catch (e) {
        statusDiv.innerHTML = '<span class="error">Invalid JSON — check your formatting and try again.</span>';
        return;
    }

    if (typeof answer !== 'object' || Array.isArray(answer)) {
        statusDiv.innerHTML = '<span class="error">Answer must be a JSON object: {"word": count, ...}</span>';
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
                answer: answer,
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
    startBtn.disabled = false;
    submitBtn.disabled = true;
}

// Event listeners
startBtn.addEventListener('click', startChallenge);
submitBtn.addEventListener('click', submitAnswer);

// Programmatic API for AI integration
window.BotCaptchaAPI = {
    // Fetch a new challenge. Returns { challenge_id, text_content, duration_ms }
    async getChallenge() {
        const response = await fetch('/api/challenge');
        return await response.json();
    },

    // Returns the currently loaded challenge (if started via the UI)
    getCurrentChallenge() {
        return currentChallenge;
    },

    // Submit a word frequency map. wordFrequencies = { word: count, ... }
    // Returns { success: bool, message: string, score: number }
    async submitChallenge(challengeId, wordFrequencies) {
        const response = await fetch('/api/submit', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                challenge_id: challengeId,
                answer: wordFrequencies,
            }),
        });
        return await response.json();
    },
};
