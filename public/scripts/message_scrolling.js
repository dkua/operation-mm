const msg_container = document.getElementById("msg-container");
const messages = msg_container.children;
let snapped_message_index = get_snapped_message_index();
let update_timeout;

msg_container.addEventListener("scrollend", _ => {
  update_timeout = setTimeout(() => snapped_message_index = get_snapped_message_index(), 50);
});
msg_container.addEventListener("scroll", _ => clearTimeout(update_timeout));
document.body.addEventListener("keydown", e => {
  if (e.key === "ArrowLeft") {
    e.preventDefault();
    scroll_messages_by(-1);
  } else if (e.key === "ArrowRight") {
    e.preventDefault();
    scroll_messages_by(1);
  }
});

/**
  * Perform a binary search to find the index of the message closest to center in view
  * @returns {number}
  */
function get_snapped_message_index() {
  const view_center = document.documentElement.clientWidth / 2;
  const width_message = messages.item(0).clientWidth;
  let index_left = 0;
  let index_right = messages.length;
  while (index_left < index_right) {
    const index_guess = Math.floor((index_left + index_right) / 2);
    const mid_offset_guess = messages.item(index_guess).getBoundingClientRect().left + width_message / 2;
    if (mid_offset_guess < view_center) {
      index_left = index_guess + 1;
    } else {
      index_right = index_guess;
    }
  }

  if (index_left >= messages.length) {
    return messages.length - 1;
  } else if (index_left === 0) {
    return index_left;
  } else {
    // The closest to center is index_left if the message was offset towards the right
    // Else it would be index_left - 1
    const mid_offset_left = messages.item(index_left).getBoundingClientRect().left + width_message / 2;
    const mid_offset_other_candidate = messages.item(index_left - 1).getBoundingClientRect().left + width_message / 2;
    if (Math.abs(mid_offset_left - view_center) < Math.abs(mid_offset_other_candidate - view_center)) {
      return index_left;
    } else {
      return index_left - 1;
    }
  }
}

/**
 * @param num {number}
 */
function scroll_messages_by(num) {
  snapped_message_index = Math.min(messages.length - 1, Math.max(0, snapped_message_index + num));
  messages.item(snapped_message_index).scrollIntoView({ behavior: "smooth", inline: "center" });
}
