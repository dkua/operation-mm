{
    const msg_container = document.getElementById("msg-container");
    /** @type {HTMLTemplateElement} */
    const messages = document.getElementById("messages-template");
    // Normally template content is cloned before placing in the DOM, but in this case
    // the contents are only needed once
    msg_container.replaceChildren(messages.content);
    const message_masonry = new Masonry(msg_container, {
        itemSelector: ".msg",
        gutter: 20,
        fitWidth: true,
        layoutInstant: true,
    });
}
