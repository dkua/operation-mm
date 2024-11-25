const msg_container = document.getElementById("msg-container");
// Set style height to current height so the container doesn't momentarily shrink to 0 when applying Masonry,
// which causes scroll position to be lost
msg_container.style.height = `${msg_container.clientHeight}px`;
const message_masonry = new Masonry(msg_container, {
    itemSelector: ".msg",
    gutter: 20,
    fitWidth: true,
    layoutInstant: true,
    initLayout: false
});
// HACK: Delay layout to next frame for Safari shenanigans (layout running before window has correct size values)
requestAnimationFrame(() => {
    message_masonry.layout();
});
