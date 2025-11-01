// Video thumbnails bar functionality
(function () {
    const video = document.getElementById('main-video');
    const thumbnailsBar = document.getElementById('video-thumbnails-bar');
    const thumbnailsScroll = document.querySelector('.thumbnails-scroll');

    if (!video || !thumbnailsBar || !thumbnailsScroll) {
        return;
    }

    let previewInfo = null;
    let thumbnailsLoaded = false;
    let currentActiveThumbnail = null;

    const revealThumbnailsBar = () => {
        thumbnailsBar.classList.remove('hidden');
        thumbnailsBar.classList.add('block');
    };

    const setLoadingMarkup = () => {
        thumbnailsScroll.innerHTML = `
            <div class="loading-thumbnails flex w-full items-center justify-center gap-3 rounded-lg border border-dashed border-slate-700 bg-slate-900/50 px-4 py-6 text-sm text-slate-300">
                <span class="h-4 w-4 animate-spin rounded-full border-2 border-emerald-400/70 border-t-transparent"></span>
                <span>Carregando miniaturas...</span>
            </div>
        `;
    };

    const setErrorMarkup = () => {
        thumbnailsScroll.innerHTML = `
            <div class="loading-thumbnails error-state flex w-full flex-col items-center justify-center gap-3 rounded-lg border border-dashed border-rose-500/40 bg-rose-500/10 px-4 py-6 text-center text-sm text-rose-200">
                <div class="error-icon text-2xl">⚠️</div>
                <div class="error-text space-y-1">
                    <div>Erro ao gerar miniaturas</div>
                    <div class="error-subtitle text-xs text-rose-100/80">Verifique se o ffmpeg está instalado e funcionando</div>
                </div>
            </div>
        `;
    };

    // Force show thumbnails bar after 2 seconds if not loaded
    setTimeout(function () {
        if (thumbnailsBar.classList.contains('hidden')) {
            console.log('Forcing thumbnails bar to show...');
            showLoadingState();
            // Try to load again
            loadPreviewInfo()
                .then(createThumbnailsBar)
                .catch(error => {
                    console.error('Error in forced loading attempt:', error);
                    showErrorState();
                });
        }
    }, 2000);

    // Load preview info and create thumbnails when video metadata is loaded
    video.addEventListener('loadedmetadata', async function () {
        console.log('Video metadata loaded, initializing thumbnails...');

        // Show loading immediately
        showLoadingState();

        try {
            await loadPreviewInfo();
            await createThumbnailsBar();
        } catch (error) {
            console.error('Error in video thumbnail initialization:', error);
            showErrorState();
        }
    });

    function showLoadingState() {
        setLoadingMarkup();
        revealThumbnailsBar();
    }

    function showErrorState() {
        setErrorMarkup();
        revealThumbnailsBar();
    }

    // Update active thumbnail when video time changes
    video.addEventListener('timeupdate', function () {
        if (!previewInfo || !thumbnailsLoaded) return;
        updateActiveThumbnail(video.currentTime);
    });

    async function loadPreviewInfo() {
        console.log('Loading preview info for:', videoPath);
        const url = `/video-previews/${videoPath}`;
        console.log('Fetching:', url);

        const response = await fetch(url);
        console.log('Response status:', response.status);

        if (!response.ok) {
            console.error('Failed to load preview info:', response.status, response.statusText);
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        previewInfo = await response.json();
        console.log('Preview info loaded:', previewInfo);
    }

    async function createThumbnailsBar() {
        console.log('Starting createThumbnailsBar...');
        if (!previewInfo || thumbnailsLoaded) {
            console.log('Skipping createThumbnailsBar - previewInfo:', !!previewInfo, 'thumbnailsLoaded:', thumbnailsLoaded);
            return;
        }

        // Use expected_count if available, otherwise estimate
        const totalEstimated = previewInfo.expected_count || Math.max(1, Math.ceil(previewInfo.duration / previewInfo.interval));
        console.log('Total estimated thumbnails:', totalEstimated);

        // Container to track created thumbnails by time
        const createdTimes = new Set();

        // Since thumbnails are already available (based on preview info), skip polling
        if (previewInfo.thumbnails && previewInfo.thumbnails.length > 0) {
            console.log('Thumbnails already available, creating elements directly...');

            // Keep the loading state while creating thumbnails
            // Clear loading and create thumbnail elements after all are ready
            const thumbnailElements = [];

            for (let i = 0; i < previewInfo.thumbnails.length; i++) {
                const thumbnail = previewInfo.thumbnails[i];
                console.log(`Creating thumbnail ${i + 1}/${previewInfo.thumbnails.length} for time ${thumbnail.time}s`);

                try {
                    const thumbnailElement = await createThumbnailElement(thumbnail);
                    thumbnailElements.push(thumbnailElement);
                    console.log(`Thumbnail element created for time ${thumbnail.time}s`);
                } catch (error) {
                    console.error(`Error creating thumbnail for time ${thumbnail.time}:`, error);
                }
            }

            // Replace loading with all thumbnails at once
            thumbnailsScroll.innerHTML = '';
            thumbnailElements.forEach(element => {
                thumbnailsScroll.appendChild(element);
            });

            thumbnailsLoaded = true;
            console.log('All thumbnails loaded successfully');
            console.log('thumbnailsScroll children count:', thumbnailsScroll.children.length);
            console.log('thumbnailsBar visible:', !thumbnailsBar.classList.contains('hidden'));
            return;
        }        // Fallback: Poll preview info until all thumbnails are generated (for live generation)
        let lastCount = 0;
        let pollAttempts = 0;
        while (true) {
            pollAttempts++;
            console.log(`Poll attempt ${pollAttempts}, lastCount: ${lastCount}`);

            try {
                const resp = await fetch(`/video-previews/${videoPath}`);
                if (resp.ok) {
                    previewInfo = await resp.json();
                    console.log(`Updated previewInfo: ${previewInfo.thumbnails.length} thumbnails available`);
                }
            } catch (err) {
                console.warn('Erro ao obter progresso das miniaturas:', err);
            }

            const generated = (previewInfo && previewInfo.thumbnails) ? previewInfo.thumbnails.length : 0;
            const total = previewInfo && previewInfo.expected_count ? previewInfo.expected_count : totalEstimated;

            // Append any new thumbnails
            if (generated > 0) {
                // Ensure thumbnailsScroll contains the thumbnail elements (remove loading UI)
                if (!thumbnailsScroll.querySelector('.thumbnail-item')) {
                    thumbnailsScroll.innerHTML = '';
                }

                for (const thumb of previewInfo.thumbnails) {
                    if (!createdTimes.has(String(thumb.time))) {
                        const el = await createThumbnailElement(thumb);
                        thumbnailsScroll.appendChild(el);
                        createdTimes.add(String(thumb.time));
                    }
                }
            }

            // Stop polling if we've reached expected total or generation seems finished
            if (generated >= total && total > 0) {
                thumbnailsLoaded = true;
                // remove loading overlay if any
                // All thumbnails generated
                break;
            }

            // if no progress for a while, break to avoid infinite loop
            if (generated == lastCount) {
                // wait a bit and try again
                await new Promise(r => setTimeout(r, 600));
            } else {
                lastCount = generated;
                await new Promise(r => setTimeout(r, 300));
            }
        }

        // final cleanup: hide loading UI if thumbnails exist
        if (thumbnailsScroll.querySelector('.thumbnail-item')) {
            // nothing else to do; thumbnails are visible
        } else {
            thumbnailsScroll.innerHTML = '<div class="loading-thumbnails flex w-full items-center justify-center rounded-lg border border-dashed border-slate-700 bg-slate-900/50 px-4 py-6 text-sm text-slate-300">Nenhuma miniatura disponível</div>';
        }
    }

    async function createThumbnailElement(thumbnail) {
        const thumbnailDiv = document.createElement('div');
        thumbnailDiv.className = 'thumbnail-item group flex w-32 flex-shrink-0 cursor-pointer flex-col gap-2 rounded-lg border border-slate-800/70 bg-slate-950/70 p-2 text-xs text-slate-300 transition hover:border-slate-600 hover:bg-slate-900/80';
        thumbnailDiv.dataset.time = thumbnail.time;

        const img = document.createElement('img');
        img.alt = `Preview em ${formatTime(thumbnail.time)}`;
        img.className = 'h-20 w-full rounded-md object-cover object-center shadow-inner shadow-black/30';

        const timeLabel = document.createElement('div');
        timeLabel.className = 'thumbnail-time text-center font-medium text-slate-200 transition';
        timeLabel.textContent = formatTime(thumbnail.time);

        thumbnailDiv.appendChild(img);
        thumbnailDiv.appendChild(timeLabel);

        // Add click handler
        thumbnailDiv.addEventListener('click', function () {
            seekToTime(thumbnail.time);
        });

        // Load thumbnail image
        try {
            console.log(`Fetching thumbnail image for time ${thumbnail.time}s...`);
            const response = await fetch(`/video-previews/${videoPath}?time=${thumbnail.time}`);
            console.log(`Image response for ${thumbnail.time}s: ${response.status}`);

            if (response.ok) {
                const blob = await response.blob();
                img.src = URL.createObjectURL(blob);
                console.log(`Image loaded successfully for ${thumbnail.time}s`);

                // Clean up blob URL when image is loaded
                img.onload = function () {
                    // Store the blob URL for later cleanup
                    img.dataset.blobUrl = img.src;
                };
            } else {
                img.src = 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTYwIiBoZWlnaHQ9IjkwIiB2aWV3Qm94PSIwIDAgMTYwIDkwIiBmaWxsPSJub25lIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPgo8cmVjdCB3aWR0aD0iMTYwIiBoZWlnaHQ9IjkwIiBmaWxsPSIjMmEyYTJhIi8+Cjx0ZXh0IHg9IjgwIiB5PSI0NSIgZmlsbD0iIzk5OTk5OSIgdGV4dC1hbmNob3I9Im1pZGRsZSIgZG9taW5hbnQtYmFzZWxpbmU9ImNlbnRyYWwiIGZvbnQtZmFtaWx5PSJBcmlhbCIgZm9udC1zaXplPSIxMiI+RXJybzwvdGV4dD4KPHN2Zz4K';
            }
        } catch (error) {
            console.warn(`Erro ao carregar miniatura para ${thumbnail.time}s:`, error);
            img.src = 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTYwIiBoZWlnaHQ9IjkwIiB2aWV3Qm94PSIwIDAgMTYwIDkwIiBmaWxsPSJub25lIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPgo8cmVjdCB3aWR0aD0iMTYwIiBoZWlnaHQ9IjkwIiBmaWxsPSIjMmEyYTJhIi8+Cjx0ZXh0IHg9IjgwIiB5PSI0NSIgZmlsbD0iIzk5OTk5OSIgdGV4dC1hbmNob3I9Im1pZGRsZSIgZG9taW5hbnQtYmFzZWxpbmU9ImNlbnRyYWwiIGZvbnQtZmFtaWx5PSJBcmlhbCIgZm9udC1zaXplPSIxMiI+RXJybzwvdGV4dD4KPHN2Zz4K';
        }

        return thumbnailDiv;
    }

    function seekToTime(time) {
        video.currentTime = time;
        updateActiveThumbnail(time);

        // Scroll the active thumbnail into view
        const activeThumbnail = document.querySelector('.thumbnail-item.active');
        if (activeThumbnail) {
            activeThumbnail.scrollIntoView({
                behavior: 'smooth',
                block: 'nearest',
                inline: 'center'
            });
        }
    }

    function updateActiveThumbnail(currentTime) {
        if (!previewInfo) return;

        // Find the closest thumbnail
        let closestThumbnail = null;
        let minDiff = Infinity;

        for (const thumbnail of previewInfo.thumbnails) {
            const diff = Math.abs(thumbnail.time - currentTime);
            if (diff < minDiff) {
                minDiff = diff;
                closestThumbnail = thumbnail;
            }
        }

        if (closestThumbnail) {
            // Remove previous active class
            if (currentActiveThumbnail) {
                currentActiveThumbnail.classList.remove('active', 'border-emerald-400/70', 'bg-emerald-500/15', 'text-emerald-200');
                const previousLabel = currentActiveThumbnail.querySelector('.thumbnail-time');
                if (previousLabel) {
                    previousLabel.classList.remove('text-emerald-200');
                }
            }

            // Add active class to closest thumbnail
            const thumbnailElement = document.querySelector(`[data-time="${closestThumbnail.time}"]`);
            if (thumbnailElement) {
                thumbnailElement.classList.add('active', 'border-emerald-400/70', 'bg-emerald-500/15', 'text-emerald-200');
                const label = thumbnailElement.querySelector('.thumbnail-time');
                if (label) {
                    label.classList.add('text-emerald-200');
                }
                currentActiveThumbnail = thumbnailElement;
            }
        }
    }

    function formatTime(seconds) {
        const hours = Math.floor(seconds / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);
        const secs = Math.floor(seconds % 60);

        if (hours > 0) {
            return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
        } else {
            return `${minutes}:${secs.toString().padStart(2, '0')}`;
        }
    }

    // Clean up blob URLs on page unload
    window.addEventListener('beforeunload', function () {
        const thumbnailImages = document.querySelectorAll('.thumbnail-item img[data-blob-url]');
        thumbnailImages.forEach(img => {
            if (img.dataset.blobUrl && img.dataset.blobUrl.startsWith('blob:')) {
                URL.revokeObjectURL(img.dataset.blobUrl);
            }
        });
    });
})();