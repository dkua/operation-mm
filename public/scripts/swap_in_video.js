/**
  * @param {string} video_id
  * @param {Element} el
  */
function swap_in_video(video_id, el) {
  const video_iframe = document.createElement('iframe');
  video_iframe.src = `https://www.youtube-nocookie.com/embed/${video_id}`;
  video_iframe.title = 'YouTube video player';
  video_iframe.allow = 'accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share';
  video_iframe.allowFullscreen = true;
  video_iframe.referrerPolicy = 'strict-origin-when-cross-origin';
  video_iframe.className = 'video';
  el.replaceWith(video_iframe);
}

for (const el of document.getElementsByClassName('video placeholder')) {
  el.addEventListener('click', () => swap_in_video(el.getAttribute('data-video_id'), el));
}
