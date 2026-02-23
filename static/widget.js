/**
 * BotCaptcha Widget — UJS Loader
 *
 * Embed on any page:
 *   <script src="https://yourdomain.com/widget.js" async defer></script>
 *   <div class="ai-captcha" data-sitekey="sk_demo_123456"></div>
 *
 * Flow:
 *   1. Widget renders a "Verify AI" button (idle — no challenge fetched yet, no timer)
 *   2. Button click (or el.aiCaptcha.start()) → fetches challenge, starts timer, opens modal
 *   3. AI agent reads challenge via el.aiCaptcha.getChallenge()
 *   4. AI solves externally, calls el.aiCaptcha.submit(wordFreqMap, sortedGridCoords)
 *   5. Server verifies → signed token returned → ai-captcha-success event fired
 *
 * Global API:
 *   window.aiCaptcha.start(el)
 *   window.aiCaptcha.getChallenge(el)   → { challenge_id, text_content, grid_coords_text, ... }
 *   window.aiCaptcha.submit(el, wordMap, gridCoords)
 *   window.aiCaptcha.getToken(el)
 *   window.aiCaptcha.reset(el)
 *
 * Events (fired on the host element):
 *   ai-captcha-ready    — challenge loaded, timer running, ready for answer
 *   ai-captcha-success  — e.detail = { token, score }
 */
(function () {
  'use strict';

  // Derive API base from this script tag's src so the widget works on any host page
  function getApiBase() {
    const scripts = document.querySelectorAll('script[src]');
    for (const s of scripts) {
      if (s.src && s.src.includes('widget.js')) {
        try { return new URL(s.src).origin; } catch (_) {}
      }
    }
    return window.location.origin;
  }

  const API_BASE = getApiBase();

  // ── Styles (shadow DOM — fully encapsulated) ──────────────────────────────────
  const STYLE = `
    :host { display: block; font-family: system-ui, -apple-system, sans-serif; }

    /* ── Inline badge (always visible) ─────────── */
    .badge {
      display: flex;
      align-items: center;
      gap: 10px;
      border: 1px solid #d1d5db;
      border-radius: 8px;
      padding: 10px 14px;
      background: #f9fafb;
      min-height: 52px;
    }
    .badge-icon { font-size: 20px; flex-shrink: 0; line-height: 1; }
    .badge-text { flex: 1; }
    .badge-label { font-size: 13px; font-weight: 600; color: #111827; }
    .badge-sub   { font-size: 11px; color: #6b7280; margin-top: 2px; }
    .badge-tag {
      font-size: 10px; font-weight: 500; padding: 2px 8px;
      border-radius: 10px; white-space: nowrap; flex-shrink: 0;
      background: #e5e7eb; color: #374151;
    }
    .badge-tag.pass { background: #d1fae5; color: #065f46; }

    .verify-btn {
      padding: 7px 14px; font-size: 12px; font-weight: 600;
      border-radius: 6px; border: 1px solid #6366f1;
      background: #6366f1; color: #fff; cursor: pointer; flex-shrink: 0;
    }
    .verify-btn:hover { background: #4f46e5; border-color: #4f46e5; }

    .spinner {
      flex-shrink: 0; width: 18px; height: 18px;
      border: 2px solid #d1d5db; border-top-color: #6366f1;
      border-radius: 50%; animation: spin .7s linear infinite;
    }
    @keyframes spin { to { transform: rotate(360deg); } }

    /* ── Modal overlay ──────────────────────────── */
    .overlay {
      display: none;
      position: fixed; inset: 0; z-index: 9999;
      background: rgba(0,0,0,.5);
      align-items: center; justify-content: center;
      padding: 16px;
    }
    .overlay.open { display: flex; }

    .dialog {
      background: #fff; border-radius: 12px;
      width: 100%; max-width: 520px;
      box-shadow: 0 20px 60px rgba(0,0,0,.3);
      overflow: hidden;
      display: flex; flex-direction: column;
    }

    /* Dialog header */
    .dialog-header {
      display: flex; align-items: center; gap: 10px;
      padding: 16px 20px 12px;
      border-bottom: 1px solid #e5e7eb;
    }
    .dialog-title { flex: 1; font-size: 15px; font-weight: 700; color: #111827; }
    .dialog-sub   { font-size: 12px; color: #6b7280; margin-top: 1px; }
    .close-btn {
      width: 28px; height: 28px; border-radius: 6px;
      border: 1px solid #e5e7eb; background: #fff;
      color: #6b7280; font-size: 16px; line-height: 1;
      cursor: pointer; display: flex; align-items: center; justify-content: center;
      flex-shrink: 0;
    }
    .close-btn:hover { background: #f3f4f6; color: #111827; }

    /* Timer bar */
    .timer-bar-wrap {
      padding: 0 20px 12px;
      border-bottom: 1px solid #e5e7eb;
    }
    .timer-row {
      display: flex; justify-content: space-between;
      font-size: 11px; color: #6b7280; margin-bottom: 5px;
    }
    .timer-val { font-weight: 600; color: #111827; }
    .timer-bar-bg {
      height: 4px; background: #e5e7eb; border-radius: 2px; overflow: hidden;
    }
    .timer-bar-fill {
      height: 100%; background: #6366f1; border-radius: 2px;
      transition: width .1s linear, background .5s;
    }
    .timer-bar-fill.urgent { background: #ef4444; }

    /* Dialog body */
    .dialog-body { padding: 16px 20px; display: flex; flex-direction: column; gap: 14px; overflow-y: auto; max-height: 60vh; }

    .section-label {
      font-size: 10px; font-weight: 700; text-transform: uppercase;
      letter-spacing: .06em; color: #9ca3af; margin-bottom: 5px;
    }
    .text-box {
      font-family: 'Menlo', 'Courier New', monospace;
      font-size: 11px; line-height: 1.7; color: #374151;
      background: #f8fafc; border: 1px solid #e2e8f0;
      border-radius: 6px; padding: 10px 12px;
      max-height: 120px; overflow-y: auto;
      white-space: pre-wrap; word-break: break-word;
    }

    /* Instructions */
    .intro-banner {
      background: #eff6ff; border: 1px solid #bfdbfe;
      border-radius: 6px; padding: 10px 12px;
      font-size: 12px; line-height: 1.6; color: #1e40af;
    }
    .intro-banner strong { font-weight: 700; }
    .puzzle-hint {
      font-size: 11px; color: #6b7280; line-height: 1.5; margin-bottom: 5px;
    }
    .puzzle-hint code {
      font-family: 'Menlo', 'Courier New', monospace;
      font-size: 10.5px; background: #f3f4f6;
      border-radius: 3px; padding: 1px 4px; color: #374151;
    }
    .code-block {
      font-family: 'Menlo', 'Courier New', monospace;
      font-size: 11px; line-height: 1.7; color: #c7d2fe;
      background: #1e1b4b; border-radius: 6px;
      padding: 10px 12px; white-space: pre; overflow-x: auto;
    }
    .code-block .cm  { color: #818cf8; }
    .code-block .cs  { color: #6ee7b7; }

    /* Dialog footer */
    .dialog-footer {
      padding: 12px 20px; border-top: 1px solid #e5e7eb;
      display: flex; align-items: center; gap: 10px;
    }
    .status-line { flex: 1; font-size: 12px; color: #6b7280; }
    .status-line.ok  { color: #059669; font-weight: 600; }
    .status-line.err { color: #dc2626; font-weight: 600; }
    .retry-btn {
      padding: 6px 14px; font-size: 12px; font-weight: 600;
      border-radius: 6px; border: 1px solid #d1d5db;
      background: #fff; color: #374151; cursor: pointer; flex-shrink: 0;
    }
    .retry-btn:hover { background: #f3f4f6; }
    .branding { font-size: 10px; color: #d1d5db; flex-shrink: 0; }
  `;

  // ── Widget class ──────────────────────────────────────────────────────────────
  class BotCaptchaWidget {
    constructor(host, sitekey, options) {
      this.host     = host;
      this.sitekey  = sitekey;
      this.callback = options.callback || null;

      this.challenge      = null;
      this.token          = null;
      this.deadline       = null;
      this._state         = 'idle';
      this._statusMsg     = '';
      this._countdownTimer = null;
      this._expiryTimer   = null;

      this._buildShadow();
    }

    // ── DOM ───────────────────────────────────────────────────────────────────
    _buildShadow() {
      const shadow = this.host.attachShadow({ mode: 'open' });

      const style = document.createElement('style');
      style.textContent = STYLE;
      shadow.appendChild(style);

      // Inline badge (always visible in the form)
      this._badgeEl = document.createElement('div');
      this._badgeEl.className = 'badge';
      shadow.appendChild(this._badgeEl);

      // Modal overlay (hidden until start())
      this._overlayEl = document.createElement('div');
      this._overlayEl.className = 'overlay';
      this._overlayEl.innerHTML = `
        <div class="dialog">
          <div class="dialog-header">
            <div>
              <div class="dialog-title">🤖 AI Verification</div>
              <div class="dialog-sub">BotCaptcha — prove you're an AI</div>
            </div>
            <button class="close-btn" title="Cancel">&times;</button>
          </div>
          <div class="timer-bar-wrap" style="display:none">
            <div class="timer-row">
              <span>Time remaining</span>
              <span class="timer-val">—</span>
            </div>
            <div class="timer-bar-bg"><div class="timer-bar-fill" style="width:100%"></div></div>
          </div>
          <div class="dialog-body">
            <div class="body-loading" style="font-size:13px;color:#6b7280;padding:8px 0;">
              Fetching challenge…
            </div>
            <div class="body-challenge" style="display:none">

              <div>
                <div class="section-label">Puzzle 1 — Word Frequency</div>
                <div class="puzzle-hint">
                  Count how many times each unique word appears in the passage.
                  Submit as a word→count map: <code>{"the": 12, "and": 8, …}</code>
                </div>
                <div class="text-box text-content"></div>
              </div>

              <div>
                <div class="section-label">Puzzle 2 — Grid Ordering</div>
                <div class="puzzle-hint">
                  Parse the natural-language description to extract grid cell coordinates.
                  Sort them into reading order (row ascending, then column ascending).
                  Submit as: <code>[{col: 3, row: 1}, {col: 0, row: 2}, …]</code>
                </div>
                <div class="text-box grid-desc"></div>
              </div>

              <div>
                <div class="section-label">How to submit</div>
                <div class="code-block"><span class="cm">// get this element</span>
<span class="cs">const</span> el = document.querySelector(<span class="cs">'.ai-captcha'</span>);
<span class="cm">// read both puzzles</span>
<span class="cs">const</span> c  = el.aiCaptcha.getChallenge();
<span class="cm">// c.text_content     → word frequency passage</span>
<span class="cm">// c.grid_coords_text → natural-language grid description</span>
<span class="cm">// submit solved answers</span>
await el.aiCaptcha.submit(wordFreqMap, sortedGridCoords);</div>
              </div>

            </div>
          </div>
          <div class="dialog-footer">
            <div class="status-line">Loading…</div>
            <div class="branding">BotCaptcha</div>
          </div>
        </div>
      `;
      shadow.appendChild(this._overlayEl);

      // Cache refs
      this._timerWrap  = this._overlayEl.querySelector('.timer-bar-wrap');
      this._timerVal   = this._overlayEl.querySelector('.timer-val');
      this._timerFill  = this._overlayEl.querySelector('.timer-bar-fill');
      this._bodyLoading  = this._overlayEl.querySelector('.body-loading');
      this._bodyChallenge = this._overlayEl.querySelector('.body-challenge');
      this._textContent  = this._overlayEl.querySelector('.text-content');
      this._gridDesc     = this._overlayEl.querySelector('.grid-desc');
      this._statusLine   = this._overlayEl.querySelector('.status-line');
      this._closeBtn     = this._overlayEl.querySelector('.close-btn');

      this._closeBtn.addEventListener('click', () => this._cancel());

      this._renderBadge();
    }

    _renderBadge() {
      const b = this._badgeEl;
      switch (this._state) {
        case 'idle':
          b.innerHTML = `
            <div class="badge-icon">🤖</div>
            <div class="badge-text">
              <div class="badge-label">AI Verification</div>
              <div class="badge-sub">Click to begin the challenge</div>
            </div>
            <button class="verify-btn">Verify AI →</button>
          `;
          b.querySelector('.verify-btn').addEventListener('click', () => this.start());
          break;

        case 'loading':
          b.innerHTML = `
            <div class="spinner"></div>
            <div class="badge-text">
              <div class="badge-label">AI Verification</div>
              <div class="badge-sub">Fetching challenge…</div>
            </div>
            <span class="badge-tag">BotCaptcha</span>
          `;
          break;

        case 'ready':
        case 'submitting':
          b.innerHTML = `
            <div class="badge-icon">🤖</div>
            <div class="badge-text">
              <div class="badge-label">AI Verification</div>
              <div class="badge-sub">${this._state === 'submitting' ? 'Verifying answer…' : 'Challenge open — awaiting answer'}</div>
            </div>
            <span class="badge-tag">BotCaptcha</span>
          `;
          break;

        case 'pass':
          b.innerHTML = `
            <div class="badge-icon">✅</div>
            <div class="badge-text">
              <div class="badge-label">Verified</div>
              <div class="badge-sub">AI identity confirmed</div>
            </div>
            <span class="badge-tag pass">Passed</span>
          `;
          break;

        case 'fail':
        case 'expired':
        case 'error':
          b.innerHTML = `
            <div class="badge-icon">${this._state === 'expired' ? '⏱️' : '❌'}</div>
            <div class="badge-text">
              <div class="badge-label">${this._state === 'expired' ? 'Expired' : 'Failed'}</div>
              <div class="badge-sub">Click to retry</div>
            </div>
            <button class="verify-btn">Retry →</button>
          `;
          b.querySelector('.verify-btn').addEventListener('click', () => this._reset());
          break;
      }
    }

    _setStatus(msg, type = '') {
      this._statusLine.textContent = msg;
      this._statusLine.className = 'status-line' + (type ? ' ' + type : '');
    }

    // ── Start / fetch ──────────────────────────────────────────────────────────
    async start() {
      if (this._state !== 'idle') return;
      this._state = 'loading';
      this._renderBadge();
      this._openModal();
      await this._fetchChallenge();
    }

    async _fetchChallenge() {
      this._bodyLoading.style.display  = 'block';
      this._bodyChallenge.style.display = 'none';
      this._timerWrap.style.display    = 'none';
      this._setStatus('Fetching challenge…');

      try {
        const url = new URL('/api/challenge', API_BASE);
        if (this.sitekey) url.searchParams.set('sitekey', this.sitekey);

        const res = await fetch(url.toString());
        if (!res.ok) {
          const body = await res.json().catch(() => ({}));
          throw new Error(body.error || `Server error ${res.status}`);
        }
        const data = await res.json();
        if (data.error) throw new Error(data.error);

        this.challenge = data;
        this.deadline  = Date.now() + data.duration_ms;

        // Populate modal content
        this._textContent.textContent = data.text_content;
        this._gridDesc.textContent    = data.grid_coords_text;
        this._bodyLoading.style.display  = 'none';
        this._bodyChallenge.style.display = 'block';
        this._timerWrap.style.display    = 'block';

        this._state = 'ready';
        this._renderBadge();
        this._setStatus('Challenge loaded — submit your answer via the API');
        this._startCountdown();

        // Notify listeners that the challenge is ready to read
        this.host.dispatchEvent(new CustomEvent('ai-captcha-ready', {
          bubbles: true,
          detail: { challenge: this.getChallenge() },
        }));

      } catch (err) {
        this._state = 'error';
        this._renderBadge();
        this._setStatus('Error: ' + err.message, 'err');
        this._bodyLoading.textContent = '⚠️ ' + err.message;
      }
    }

    // ── Timer ──────────────────────────────────────────────────────────────────
    _startCountdown() {
      const total = this.challenge.duration_ms;

      const tick = () => {
        const remaining = Math.max(0, this.deadline - Date.now());
        const pct = (remaining / total) * 100;

        this._timerVal.textContent = (remaining / 1000).toFixed(1) + 's';
        this._timerFill.style.width = pct + '%';

        if (pct < 25) {
          this._timerFill.classList.add('urgent');
        }

        if (remaining <= 0) {
          clearInterval(this._countdownTimer);
          if (this._state === 'ready') {
            this._state = 'expired';
            this._renderBadge();
            this._setStatus('Time expired — click Retry for a new challenge', 'err');
            this._addRetryButton();
          }
        }
      };

      tick();
      this._countdownTimer = setInterval(tick, 100);
    }

    _stopCountdown() {
      if (this._countdownTimer) {
        clearInterval(this._countdownTimer);
        this._countdownTimer = null;
      }
    }

    _addRetryButton() {
      const footer = this._overlayEl.querySelector('.dialog-footer');
      if (footer.querySelector('.retry-btn')) return;
      const btn = document.createElement('button');
      btn.className = 'retry-btn';
      btn.textContent = 'New Challenge';
      btn.addEventListener('click', () => this._reset());
      footer.insertBefore(btn, this._statusLine.nextSibling);
    }

    // ── Modal open/close ───────────────────────────────────────────────────────
    _openModal() {
      this._overlayEl.classList.add('open');
      // Remove any stale retry button
      const old = this._overlayEl.querySelector('.retry-btn');
      if (old) old.remove();
    }

    _closeModal() {
      this._overlayEl.classList.remove('open');
    }

    _cancel() {
      // Closing the modal without submitting returns to idle
      this._stopCountdown();
      this.challenge = null;
      this.deadline  = null;
      this._state    = 'idle';
      this._renderBadge();
      this._closeModal();
    }

    // ── Public API ─────────────────────────────────────────────────────────────

    /** Returns the challenge data or null if not yet loaded. */
    getChallenge() {
      return this.challenge ? Object.assign({}, this.challenge) : null;
    }

    /** Returns the verified token string, or null. */
    getToken() {
      return this.token;
    }

    /**
     * Submit answers. Called by the AI agent after solving the challenge.
     * @param {Object} wordAnswer  { word: count, ... }
     * @param {Array}  gridAnswer  [{ col, row }, ...] sorted by row asc, col asc
     * @returns {Promise<{ success, token?, score?, message }>}
     */
    async submit(wordAnswer, gridAnswer) {
      if (!this.challenge) throw new Error('No challenge loaded — call start() first');
      if (this._state !== 'ready') throw new Error(`Widget state is '${this._state}', expected 'ready'`);
      if (Date.now() > this.deadline) {
        this._state = 'expired';
        this._renderBadge();
        this._setStatus('Time expired — click Retry for a new challenge', 'err');
        this._addRetryButton();
        throw new Error('Challenge window has expired');
      }

      this._stopCountdown();
      this._state = 'submitting';
      this._renderBadge();
      this._setStatus('Submitting answer…');

      try {
        const res = await fetch(new URL('/api/submit', API_BASE).toString(), {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            challenge_id: this.challenge.challenge_id,
            answer: wordAnswer,
            grid_answer: gridAnswer,
          }),
        });

        const result = await res.json();

        if (result.success && result.token) {
          this.token = result.token;
          this._state = 'pass';
          this._renderBadge();
          this._setStatus('✓ Verified! AI identity confirmed.', 'ok');
          setTimeout(() => this._closeModal(), 1200);
          this._dispatchSuccess(result.token, result.score);
          return { success: true, token: result.token, score: result.score };
        } else {
          this._state = 'fail';
          this._renderBadge();
          this._setStatus('Failed: ' + (result.message || 'Score too low'), 'err');
          this._addRetryButton();
          return { success: false, message: result.message, score: result.score };
        }
      } catch (err) {
        this._state = 'error';
        this._renderBadge();
        this._setStatus('Error: ' + err.message, 'err');
        this._addRetryButton();
        throw err;
      }
    }

    // ── Token delivery ─────────────────────────────────────────────────────────
    _dispatchSuccess(token, score) {
      // Inject hidden input into nearest ancestor <form>
      const form = this.host.closest('form');
      if (form) {
        let hidden = form.querySelector('input[name="ai-captcha-response"]');
        if (!hidden) {
          hidden = document.createElement('input');
          hidden.type = 'hidden';
          hidden.name = 'ai-captcha-response';
          form.appendChild(hidden);
        }
        hidden.value = token;
      }

      // Fire CustomEvent on host element
      this.host.dispatchEvent(new CustomEvent('ai-captcha-success', {
        bubbles: true,
        detail: { token, score },
      }));

      // Call named global callback
      if (this.callback && typeof window[this.callback] === 'function') {
        try { window[this.callback](token, score); } catch (_) {}
      }
    }

    // ── Reset ──────────────────────────────────────────────────────────────────
    _reset() {
      this._stopCountdown();
      this.challenge = null;
      this.token     = null;
      this.deadline  = null;
      this._state    = 'loading';
      this._renderBadge();
      // Remove stale retry button
      const old = this._overlayEl.querySelector('.retry-btn');
      if (old) old.remove();
      this._fetchChallenge();
    }
  }

  // ── Auto-initialise ───────────────────────────────────────────────────────────
  function initAll() {
    document.querySelectorAll('.ai-captcha[data-sitekey]').forEach(function (el) {
      if (el._aiCaptchaWidget) return;
      const widget = new BotCaptchaWidget(el, el.dataset.sitekey, {
        callback: el.dataset.callback || null,
      });
      el._aiCaptchaWidget = widget;
      el.aiCaptcha = {
        start:        ()     => widget.start(),
        getChallenge: ()     => widget.getChallenge(),
        getToken:     ()     => widget.getToken(),
        submit:       (w, g) => widget.submit(w, g),
        reset:        ()     => widget._reset(),
      };
    });
  }

  // ── Global window.aiCaptcha ───────────────────────────────────────────────────
  window.aiCaptcha = {
    start:        (el)         => el._aiCaptchaWidget && el._aiCaptchaWidget.start(),
    reset:        (el)         => el._aiCaptchaWidget && el._aiCaptchaWidget._reset(),
    getToken:     (el)         => el._aiCaptchaWidget ? el._aiCaptchaWidget.getToken() : null,
    getChallenge: (el)         => el._aiCaptchaWidget ? el._aiCaptchaWidget.getChallenge() : null,
    submit:       (el, w, g)   => {
      if (!el._aiCaptchaWidget) throw new Error('No BotCaptcha widget found on element');
      return el._aiCaptchaWidget.submit(w, g);
    },
  };

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initAll);
  } else {
    initAll();
  }
})();
