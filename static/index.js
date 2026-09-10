document.addEventListener('DOMContentLoaded', () => {
    const timeline = document.querySelector('#timeline');
    const popularTimelineButton = document.querySelector('#popular-timeline-button');
    const recentTimelineButton = document.querySelector('#recent-timeline-button');

    popularTimelineButton.addEventListener('click', () => {
        timeline.setAttribute("hx-get", "/timeline/popular");
        popularTimelineButton.style.display = 'none';
        recentTimelineButton.style.removeProperty("display");
    });
    recentTimelineButton.addEventListener('click', () => {
        timeline.setAttribute("hx-get", "/timeline");
        recentTimelineButton.style.display = 'none';
        popularTimelineButton.style.removeProperty("display");
    });

    setupAltPanel();
    setupViewer();
});

// The timeline is swapped wholesale by htmx every few minutes, so every handler
// below is delegated on document rather than bound to the gallery elements.

function altTextOf(thumbnail) {
    const image = thumbnail.querySelector('img');
    return (image && image.getAttribute('alt')) || '';
}

function setupAltPanel() {
    const panel = document.querySelector('#alt-panel');
    const text = panel.querySelector('.alt-panel__text');
    let openBadge = null;

    const close = () => {
        if (!openBadge) {
            return;
        }
        openBadge.setAttribute('aria-expanded', 'false');
        openBadge = null;
        panel.hidden = true;
        document.body.append(panel);
    };

    const open = (badge) => {
        const item = badge.closest('.media-gallery__item');
        text.textContent = altTextOf(item.querySelector('.media-gallery__item-thumbnail'));
        badge.setAttribute('aria-expanded', 'true');
        openBadge = badge;
        item.append(panel);
        panel.hidden = false;
    };

    document.addEventListener('click', (event) => {
        const badge = event.target.closest('.media-gallery__alt__label');
        if (badge) {
            const wasOpen = badge === openBadge;
            close();
            if (!wasOpen) {
                open(badge);
            }
            return;
        }
        if (event.target.closest('.alt-panel__close') || !event.target.closest('#alt-panel')) {
            close();
        }
    });

    document.addEventListener('keydown', (event) => {
        if (event.key === 'Escape') {
            close();
        }
    });

    document.body.addEventListener('htmx:beforeSwap', close);
}

function setupViewer() {
    const viewer = document.querySelector('#viewer');
    const image = viewer.querySelector('.viewer__image');
    const caption = viewer.querySelector('.viewer__caption');
    const previous = viewer.querySelector('.viewer__nav--prev');
    const next = viewer.querySelector('.viewer__nav--next');
    let attachments = [];
    let index = 0;

    const show = (position) => {
        if (attachments.length === 0) {
            return;
        }
        index = (position + attachments.length) % attachments.length;
        image.src = attachments[index].url;
        image.alt = attachments[index].description;
        caption.textContent = attachments[index].description;
    };

    document.addEventListener('click', (event) => {
        const thumbnail = event.target.closest('.media-gallery__item-thumbnail');
        if (!thumbnail || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) {
            return;
        }
        event.preventDefault();
        // Snapshot the gallery as data: an htmx swap while the viewer is open
        // would otherwise leave us navigating detached elements.
        attachments = [...thumbnail.closest('.media-gallery')
            .querySelectorAll('.media-gallery__item-thumbnail')]
            .map((item) => ({url: item.getAttribute('href'), description: altTextOf(item)}));
        previous.hidden = attachments.length < 2;
        next.hidden = attachments.length < 2;
        show(attachments.findIndex((item) => item.url === thumbnail.getAttribute('href')));
        viewer.showModal();
    });

    // The figure fills the dialog, so its own padding is the clickable "backdrop".
    // Matching on the target itself keeps the image, caption and nav buttons inert.
    viewer.addEventListener('click', (event) => {
        if (event.target.closest('.viewer__close')
            || event.target === viewer
            || event.target.classList.contains('viewer__figure')) {
            viewer.close();
        }
    });

    previous.addEventListener('click', () => show(index - 1));
    next.addEventListener('click', () => show(index + 1));

    viewer.addEventListener('keydown', (event) => {
        if (event.key === 'ArrowLeft') {
            show(index - 1);
        } else if (event.key === 'ArrowRight') {
            show(index + 1);
        }
    });

    // Drop the reference to the full-resolution bitmap so it can be reclaimed.
    viewer.addEventListener('close', () => {
        image.removeAttribute('src');
        caption.textContent = '';
        attachments = [];
    });
}
