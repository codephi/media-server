(function () {
    function getParam(name) {
        const url = new URL(window.location.href);
        return url.searchParams.get(name) || '';
    }

    function prefillSearchInput() {
        const q = getParam('q');
        if (!q) return;
        const input = document.querySelector('#global-search-form input[name="q"]');
        if (input) {
            input.value = q;
        }
    }

    function formatItemRow(item) {
        const link = item.is_dir ? `/browse/${item.encoded_path}` : `/file/${item.encoded_path}`;
        const typeLabel = item.is_dir ? 'Diretório' : 'Arquivo';
        const row = document.createElement('a');
        row.href = link;
        row.className = 'flex items-center justify-between gap-4 px-4 py-3 transition hover:bg-slate-900';

        const left = document.createElement('div');
        left.className = 'min-w-0';

        const nameEl = document.createElement('div');
        nameEl.className = 'truncate font-medium text-slate-100';
        nameEl.textContent = item.name;

        const pathEl = document.createElement('div');
        pathEl.className = 'truncate text-xs text-slate-400';
        pathEl.textContent = item.rel_path;

        left.appendChild(nameEl);
        left.appendChild(pathEl);

        const right = document.createElement('div');
        right.className = 'flex shrink-0 items-center gap-6 text-sm text-slate-300';

        const kind = document.createElement('span');
        kind.className = 'rounded-full border border-slate-700/80 px-2 py-0.5 text-xs';
        kind.textContent = typeLabel;

        const size = document.createElement('span');
        size.textContent = item.size;

        const mod = document.createElement('span');
        mod.className = 'hidden sm:inline';
        mod.textContent = item.modified;

        right.appendChild(kind);
        right.appendChild(size);
        right.appendChild(mod);

        row.appendChild(left);
        row.appendChild(right);

        return row;
    }

    async function run() {
        // Preenche o campo de busca do header quando houver q
        prefillSearchInput();
        const container = document.getElementById('search-results');
        const placeholder = document.getElementById('search-placeholder');
        const summary = document.getElementById('search-summary');
        if (!container) return;

        const q = getParam('q').trim();
        if (!q) {
            summary.textContent = '';
            return;
        }

        // loading state
        if (placeholder) {
            placeholder.textContent = `Procurando por "${q}"...`;
        }

        try {
            const t0 = performance.now();
            const resp = await fetch(`/search?q=${encodeURIComponent(q)}`);
            if (!resp.ok) {
                throw new Error(`Erro ${resp.status}`);
            }
            const data = await resp.json();
            const t1 = performance.now();

            // reset
            container.innerHTML = '';

            if (!Array.isArray(data) || data.length === 0) {
                const empty = document.createElement('div');
                empty.className = 'px-4 py-10 text-center text-sm text-slate-400';
                empty.textContent = 'Nenhum resultado encontrado';
                container.appendChild(empty);
            } else {
                data.forEach(item => container.appendChild(formatItemRow(item)));
            }

            const ms = Math.max(1, Math.round(t1 - t0));
            summary.textContent = `${data.length} resultado(s) em ${ms} ms`;
        } catch (err) {
            container.innerHTML = '';
            const errEl = document.createElement('div');
            errEl.className = 'px-4 py-10 text-center text-sm text-rose-300';
            errEl.textContent = 'Erro ao buscar. Tente novamente.';
            container.appendChild(errEl);
            summary.textContent = '';
            console.error('Search error', err);
        }
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', run);
    } else {
        run();
    }
})();