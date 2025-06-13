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
});
