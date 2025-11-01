// Image viewer with zoom and pan
(function() {
    const img = document.getElementById('viewer-image');
    if (!img) return;
    
    let scale = 1;
    let panning = false;
    let pointX = 0;
    let pointY = 0;
    let start = { x: 0, y: 0 };
    
    img.style.transformOrigin = '0 0';
    img.style.cursor = 'zoom-in';
    
    // Zoom with wheel
    img.addEventListener('wheel', function(e) {
        e.preventDefault();
        
        const xs = (e.clientX - pointX) / scale;
        const ys = (e.clientY - pointY) / scale;
        
        if (e.deltaY < 0) {
            scale *= 1.2; // Zoom in
        } else {
            scale /= 1.2; // Zoom out
        }
        
        scale = Math.min(Math.max(0.5, scale), 4);
        
        pointX = e.clientX - xs * scale;
        pointY = e.clientY - ys * scale;
        
        img.style.transform = `translate(${pointX}px, ${pointY}px) scale(${scale})`;
        
        // Update cursor
        img.style.cursor = scale > 1 ? 'move' : 'zoom-in';
    });
    
    // Pan with mouse
    img.addEventListener('mousedown', function(e) {
        if (scale <= 1) return;
        e.preventDefault();
        start = { x: e.clientX - pointX, y: e.clientY - pointY };
        panning = true;
    });
    
    img.addEventListener('mousemove', function(e) {
        if (!panning) return;
        e.preventDefault();
        pointX = e.clientX - start.x;
        pointY = e.clientY - start.y;
        img.style.transform = `translate(${pointX}px, ${pointY}px) scale(${scale})`;
    });
    
    img.addEventListener('mouseup', function(e) {
        panning = false;
    });
    
    img.addEventListener('mouseleave', function(e) {
        panning = false;
    });
    
    // Double-click to reset
    img.addEventListener('dblclick', function(e) {
        e.preventDefault();
        scale = 1;
        pointX = 0;
        pointY = 0;
        img.style.transform = '';
        img.style.cursor = 'zoom-in';
    });
})();