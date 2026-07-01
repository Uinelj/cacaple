import init, {
    init as wasmInit,
    get_target,
    is_valid_move,
    get_distance,
    get_start_word,
    get_start_word_distance,
    get_day_number,
    get_optimal_path,
    word_count,
    validate_error,
    is_in_word_list,
} from './pkg/pipiple.js';

// ============================================================
// State
// ============================================================

let wasm;
let guesses = [];
let currentWord = '';
let gameOver = false;
let target = '';

// ============================================================
// localStorage helpers
// ============================================================

function lsGet(key, fallback) {
    try {
        const v = localStorage.getItem(key);
        return v !== null ? JSON.parse(v) : fallback;
    } catch { return fallback; }
}

function lsSet(key, value) {
    localStorage.setItem(key, JSON.stringify(value));
}

function getGuesses() { return lsGet('pipiple_guesses', []); }
function setGuesses(g) { lsSet('pipiple_guesses', g); }
function getGames() { return lsGet('pipiple_games', {}); }
function getStreak() { return lsGet('pipiple_streak', 0); }
function setStreak(s) { lsSet('pipiple_streak', s); }
function getBestStreak() { return lsGet('pipiple_bestStreak', 0); }
function setBestStreak(s) { lsSet('pipiple_bestStreak', s); }
function getTimeLastPlayed() { return lsGet('pipiple_dateLastPlayed', 0); }
function getTimeLastWon() { return lsGet('pipiple_dateLastWon', 0); }

function isToday(ts) {
    return get_day_number(ts) === get_day_number(Date.now());
}
function isYesterday(ts) {
    return get_day_number(ts) === get_day_number(Date.now()) - 1;
}

function winGame(extraGuesses) {
    const games = getGames();
    const key = String(extraGuesses);
    games[key] = (games[key] || 0) + 1;
    lsSet('pipiple_games', games);

    const streak = getStreak() + 1;
    setStreak(streak);
    if (streak > getBestStreak()) setBestStreak(streak);
    lsSet('pipiple_dateLastWon', Date.now());
}

// ============================================================
// Rendering
// ============================================================

function makeRow(word, opts = {}) {
    const row = document.createElement('div');
    row.className = 'Row';
    for (let i = 0; i < 4; i++) {
        const box = document.createElement('div');
        const letter = word[i] || '';
        const isHighlight = !opts.suppressHighlight && letter.toLowerCase() === target[i];
        box.className = 'Box' + (isHighlight ? ' highlight' : '') + (opts.isCurrent && letter ? ' current' : '');
        box.textContent = letter.toUpperCase();
        row.appendChild(box);
    }
    return row;
}

function renderGrid() {
    const container = document.getElementById('RowContainer');
    container.innerHTML = '';
    container.className = 'RowContainer' + (gameOver ? ' jump' : '');

    for (const word of guesses) {
        container.appendChild(makeRow(word));
    }

    if (!gameOver) {
        const currentPadded = currentWord.padEnd(4, ' ').slice(0, 4);
        container.appendChild(makeRow(currentPadded, { isCurrent: true, suppressHighlight: true }));
    }

    // Scroll to bottom
    const sc = document.getElementById('ScrollContainer');
    requestAnimationFrame(() => {
        sc.scrollTop = sc.scrollHeight;
    });
}

function showError(msg) {
    const el = document.getElementById('ErrorMessage');
    el.textContent = msg;
    el.classList.add('visible');
    setTimeout(() => {
        el.textContent = '';
        el.classList.remove('visible');
    }, 1200);
}

// ============================================================
// Keyboard
// ============================================================

const KEYBOARD_ROWS = [
    ['a', 'z', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'],
    ['q', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', 'm'],
    ['Enter', 'w', 'x', 'c', 'v', 'b', 'n', '⌫'],
];

function buildKeyboard() {
    const kb = document.getElementById('Keyboard');
    kb.innerHTML = '';
    for (const row of KEYBOARD_ROWS) {
        const rowEl = document.createElement('div');
        rowEl.className = 'KeyboardRow';
        for (const key of row) {
            const btn = document.createElement('button');
            btn.className = 'Key' + (key === 'Enter' || key === '⌫' ? ' wide' : '');
            btn.textContent = key === 'Enter' ? '↵' : key === '⌫' ? '⌫' : key.toUpperCase();
            btn.addEventListener('click', () => handleKey(key));
            // Prevent focus stealing (keeps physical keyboard working)
            btn.addEventListener('mousedown', e => e.preventDefault());
            rowEl.appendChild(btn);
        }
        kb.appendChild(rowEl);
    }
}

// ============================================================
// Game logic
// ============================================================

function handleKey(key) {
    if (gameOver) return;

    if (key === 'Backspace' || key === '⌫') {
        if (currentWord.length > 0) {
            currentWord = currentWord.slice(0, -1);
            renderGrid();
        }
        return;
    }

    if (key === 'Enter') {
        submitWord();
        return;
    }

    // Single letter
    if (/^[a-zA-Z]$/.test(key) && currentWord.length < 4) {
        currentWord += key.toLowerCase();
        renderGrid();
        // Pop animation on last box
        const boxes = document.querySelectorAll('.Row:last-child .Box.current');
        const lastFilled = [...boxes].filter(b => b.textContent.trim());
        if (lastFilled.length) {
            const b = lastFilled[lastFilled.length - 1];
            b.classList.add('pop');
            setTimeout(() => b.classList.remove('pop'), 150);
        }
    }
}

function submitWord() {
    if (currentWord.length !== 4) return;

    const previous = guesses[guesses.length - 1] || '';
    const error = validate_error(currentWord, previous);
    if (error) {
        showError(error);
        currentWord = '';
        renderGrid();
        return;
    }

    guesses.push(currentWord);
    setGuesses(guesses);

    if (currentWord.toLowerCase() === target) {
        gameOver = true;
        const par = get_distance(guesses[0]);
        const moves = guesses.length - 1;
        const extra = moves - par;
        winGame(extra);
        renderGrid();
        startEmojiRain();
        setTimeout(() => showStatsModal(), 1500);
    }

    currentWord = '';
    renderGrid();
}

// ============================================================
// Emoji Rain
// ============================================================

function startEmojiRain() {
    const container = document.getElementById('EmojiRain');
    container.innerHTML = '';
    const count = 30;
    for (let i = 0; i < count; i++) {
        const drop = document.createElement('div');
        drop.className = 'EmojiRaindrop';
        drop.textContent = '🚽';
        drop.style.left = `${Math.random() * 100}%`;
        drop.style.animationDelay = `${Math.random() * 2}s`;
        drop.style.animationDuration = `${2 + Math.random() * 2}s`;
        drop.style.transform = `rotate(${Math.random() * 30 - 15}deg)`;
        container.appendChild(drop);
    }
    setTimeout(() => { container.innerHTML = ''; }, 5000);
}

// ============================================================
// Modals
// ============================================================

function openModal(id) {
    document.getElementById(id).classList.remove('hide');
}

function closeModal(id) {
    document.getElementById(id).classList.add('hide');
}

function setupModals() {
    // Close on backdrop click or close button
    document.querySelectorAll('.Modal').forEach(modal => {
        modal.querySelector('.ModalBackdrop').addEventListener('click', () => {
            modal.classList.add('hide');
        });
        modal.querySelector('.ModalCloseButton').addEventListener('click', () => {
            modal.classList.add('hide');
        });
    });

    document.getElementById('btn-help').addEventListener('click', () => openModal('modal-help'));
    document.getElementById('btn-stats').addEventListener('click', () => showStatsModal());
    document.getElementById('btn-yesterday').addEventListener('click', () => showYesterdayModal());
}

// ============================================================
// Stats Modal
// ============================================================

function showStatsModal() {
    const container = document.getElementById('stats-content');
    const savedGuesses = getGuesses();
    const isFinished = savedGuesses.length > 0 && savedGuesses[savedGuesses.length - 1]?.toLowerCase() === target;
    const par = savedGuesses.length > 0 ? get_distance(savedGuesses[0]) : 0;
    const moves = savedGuesses.length - 1;

    let html = '';

    if (isFinished) {
        html += `<div class="Results">
            Vous avez utilisé <b>${moves} coup${moves > 1 ? 's' : ''}</b>.<br>
            Le chemin optimal était de <b>${par} coup${par > 1 ? 's' : ''}</b>.
        </div>`;

        // Copy button
        html += `<div style="text-align:center">
            <button class="copy-btn" id="copy-results">Copier les résultats</button>
            <div class="copy-feedback" id="copy-feedback"></div>
        </div>`;
    }

    // Totals
    const games = getGames();
    let totalWins = 0, totalExtra = 0;
    for (const [extra, count] of Object.entries(games)) {
        totalWins += count;
        totalExtra += parseInt(extra) * count;
    }
    const avg = totalWins > 0 ? (totalExtra / totalWins).toFixed(1) : '0';

    html += `<div class="Totals">
        <b>Statistiques</b>
        <div class="Entries">
            <div class="Entry"><h2>${totalWins}</h2><p>Victoires</p></div>
            <div class="Entry"><h2>${avg}</h2><p>Moy. coups<br>en trop</p></div>
            <div class="Entry"><h2>${getStreak()}</h2><p>Série<br>actuelle</p></div>
            <div class="Entry"><h2>${getBestStreak()}</h2><p>Meilleure<br>série</p></div>
        </div>
    </div>`;

    // Histogram
    if (totalWins > 0) {
        const maxExtra = Math.max(...Object.keys(games).map(Number));
        const maxCount = Math.max(...Object.values(games));
        html += `<div class="Histogram"><b>Distribution</b>`;
        for (let e = 0; e <= maxExtra; e++) {
            const count = games[String(e)] || 0;
            const width = Math.max(8, (count / maxCount) * 100);
            const isCurrent = isFinished && (moves - par) === e;
            html += `<div class="HistogramRow">
                <span class="HistogramLabel">+${e}</span>
                <div class="HistogramBar${isCurrent ? ' current' : ''}" style="width:${width}%">${count}</div>
            </div>`;
        }
        html += `</div>`;
    }

    // Countdown
    html += `<div class="Countdown">
        Prochain puzzle dans<br>
        <span class="time" id="countdown-time"></span>
    </div>`;

    container.innerHTML = html;

    // Wire copy button
    if (isFinished) {
        document.getElementById('copy-results').addEventListener('click', copyResults);
    }

    // Start countdown
    updateCountdown();

    openModal('modal-stats');
}

function copyResults() {
    const savedGuesses = getGuesses();
    const par = get_distance(savedGuesses[0]);
    const moves = savedGuesses.length - 1;
    const dayNum = get_day_number(getTimeLastPlayed() || Date.now());

    let text = `Pipiple #${dayNum} 🚽 ${moves}/${par}\n`;
    for (const word of savedGuesses) {
        for (let i = 0; i < 4; i++) {
            text += word[i]?.toLowerCase() === target[i] ? '🟨' : '⬜';
        }
        text += '\n';
    }

    navigator.clipboard.writeText(text).then(() => {
        const fb = document.getElementById('copy-feedback');
        fb.textContent = 'Copié ! ✓';
        fb.className = 'copy-feedback success';
        setTimeout(() => { fb.textContent = ''; }, 2000);
    }).catch(() => {
        const fb = document.getElementById('copy-feedback');
        fb.textContent = 'Erreur de copie';
        fb.className = 'copy-feedback error';
    });
}

function updateCountdown() {
    const el = document.getElementById('countdown-time');
    if (!el) return;

    const now = Date.now();
    const epoch = 1751356800000;
    const dayMs = 86400000;
    const currentDay = Math.floor((now - epoch) / dayMs);
    const nextChange = epoch + (currentDay + 1) * dayMs;
    const diff = nextChange - now;

    if (diff <= 0) {
        el.textContent = 'Maintenant !';
        return;
    }

    const h = Math.floor(diff / 3600000);
    const m = Math.floor((diff % 3600000) / 60000);
    const s = Math.floor((diff % 60000) / 1000);
    el.textContent = `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;

    setTimeout(updateCountdown, 1000);
}

// ============================================================
// Yesterday Modal
// ============================================================

function showYesterdayModal() {
    const dayNum = get_day_number(Date.now());
    if (dayNum === 0) {
        document.getElementById('yesterday-content').innerHTML = '<p>Pas encore d\'hier !</p>';
        openModal('modal-yesterday');
        return;
    }

    // Get yesterday's epoch ms
    const epoch = 1751356800000;
    const dayMs = 86400000;
    const yesterdayMs = epoch + (dayNum - 1) * dayMs + 1;

    const yesterdayWord = get_start_word(yesterdayMs);
    const yesterdayDist = get_start_word_distance(yesterdayMs);
    const pathJson = get_optimal_path(yesterdayWord);
    const path = JSON.parse(pathJson);

    document.getElementById('yesterday-title').textContent =
        `Hier — #${dayNum - 1} : ${yesterdayWord.toUpperCase()}`;

    let html = `<p>Le chemin le plus court hier était de <b>${yesterdayDist} coup${yesterdayDist > 1 ? 's' : ''}</b>.</p>`;

    for (const word of path) {
        const row = makeRow(word);
        html += row.outerHTML;
    }

    document.getElementById('yesterday-content').innerHTML = html;
    openModal('modal-yesterday');
}

// ============================================================
// Game initialization
// ============================================================

function initGame() {
    const now = Date.now();
    const todayStart = get_start_word(now);
    const dayNum = get_day_number(now);

    // Update subheader
    document.getElementById('subheader').textContent = `#${dayNum} : ${todayStart.toUpperCase()} → PIPI`;

    const savedGuesses = getGuesses();
    const lastPlayed = getTimeLastPlayed();

    // Check if we need to start a new game
    if (!isToday(lastPlayed) || savedGuesses.length === 0 || savedGuesses[0] !== todayStart) {
        // New game
        guesses = [todayStart];
        setGuesses(guesses);

        // Reset streak if didn't win yesterday
        if (!isYesterday(getTimeLastWon()) && !isToday(getTimeLastWon())) {
            setStreak(0);
        }
    } else {
        guesses = savedGuesses;
    }

    lsSet('pipiple_dateLastPlayed', Date.now());

    // Check if already won
    if (guesses.length > 0 && guesses[guesses.length - 1].toLowerCase() === target) {
        gameOver = true;
    }

    currentWord = '';
    renderGrid();
}

// ============================================================
// Boot
// ============================================================

async function boot() {
    await init();
    wasmInit();

    target = get_target().toLowerCase();

    console.log(`Pipiple loaded — ${word_count()} words, target: ${target.toUpperCase()}`);

    buildKeyboard();
    setupModals();
    initGame();

    // Physical keyboard
    window.addEventListener('keydown', (e) => {
        if (!document.querySelector('.Modal:not(.hide)')) {
            if (e.key === 'Backspace' || e.key === 'Enter' || /^[a-zA-Z]$/.test(e.key)) {
                e.preventDefault();
                handleKey(e.key);
            }
        }
        // Close modal on Escape
        if (e.key === 'Escape') {
            document.querySelectorAll('.Modal:not(.hide)').forEach(m => m.classList.add('hide'));
        }
    });

    // Show help on first visit
    if (!lsGet('pipiple_hasVisited', false)) {
        lsSet('pipiple_hasVisited', true);
        openModal('modal-help');
    }
}

boot();
