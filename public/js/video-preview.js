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

    // Force show thumbnails bar after 2 seconds if not loaded
    setTimeout(function () {
        if (thumbnailsBar.style.display === 'none') {
            console.log('Forcing thumbnails bar to show...');
            thumbnailsBar.style.display = 'block';
            if (!thumbnailsLoaded) {
                thumbnailsScroll.innerHTML = '<div class="loading-thumbnails">Carregando miniaturas...</div>';
                // Try to load again
                loadPreviewInfo().then(createThumbnailsBar).catch(console.error);
            }
        }
    }, 2000);

    // Load preview info and create thumbnails when video metadata is loaded
    video.addEventListener('loadedmetadata', async function () {
        console.log('Video metadata loaded, initializing thumbnails...');
        try {
            await loadPreviewInfo();
            await createThumbnailsBar();
            thumbnailsBar.style.display = 'block';
            console.log('Thumbnails bar should now be visible');
        } catch (error) {
            console.warn('Não foi possível carregar miniaturas:', error);
            // Show the bar anyway with an error message
            thumbnailsScroll.innerHTML = '<div class="loading-thumbnails">Erro ao carregar miniaturas de preview</div>';
            thumbnailsBar.style.display = 'block';
        }
    });

    // Update active thumbnail when video time changes
    video.addEventListener('timeupdate', function () {
        if (!previewInfo || !thumbnailsLoaded) return;
        updateActiveThumbnail(video.currentTime);
    });

    async function loadPreviewInfo() {
        const response = await fetch(`/video-previews/${videoPath}`);
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }
        previewInfo = await response.json();
    }

    async function createThumbnailsBar() {
        if (!previewInfo || thumbnailsLoaded) return;

        // Show loading state
        thumbnailsScroll.innerHTML = `
            <div class="loading-thumbnails">
                <div class="spinner"></div>
                Carregando miniaturas...
            </div>
        `;

        // Clear loading and create thumbnail elements
        thumbnailsScroll.innerHTML = '';

        for (const thumbnail of previewInfo.thumbnails) {
            const thumbnailElement = await createThumbnailElement(thumbnail);
            thumbnailsScroll.appendChild(thumbnailElement);
        }

        thumbnailsLoaded = true;
    }

    async function createThumbnailElement(thumbnail) {
        const thumbnailDiv = document.createElement('div');
        thumbnailDiv.className = 'thumbnail-item';
        thumbnailDiv.dataset.time = thumbnail.time;

        const img = document.createElement('img');
        img.alt = `Preview em ${formatTime(thumbnail.time)}`;

        const timeLabel = document.createElement('div');
        timeLabel.className = 'thumbnail-time';
        timeLabel.textContent = formatTime(thumbnail.time);

        thumbnailDiv.appendChild(img);
        thumbnailDiv.appendChild(timeLabel);

        // Add click handler
        thumbnailDiv.addEventListener('click', function () {
            seekToTime(thumbnail.time);
        });

        // Load thumbnail image
        try {
            const response = await fetch(`/video-previews/${videoPath}?time=${thumbnail.time}`);
            if (response.ok) {
                const blob = await response.blob();
                img.src = URL.createObjectURL(blob);

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
                currentActiveThumbnail.classList.remove('active');
            }

            // Add active class to closest thumbnail
            const thumbnailElement = document.querySelector(`[data-time="${closestThumbnail.time}"]`);
            if (thumbnailElement) {
                thumbnailElement.classList.add('active');
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