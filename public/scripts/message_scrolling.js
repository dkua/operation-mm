const msg_container = document.getElementById("msg-container");
const messages = msg_container.children;
/** @type {Element | null} */
let visible_message;
/** @type {Element | null} */
let snapped_message;
let update_timeout;

// Work around Safari not having scrollend event
if ("onscrollend" in msg_container) {
  // We don't want to update the index immediately on scrollend because with smooth scrolling,
  // consecutive clicks of the buttons or arrow keys will actually stop the current scrolling
  // and then start scrolling again (at least on Chrome), thus firing a scrollend event when we
  // want to keep the index on whatever the previous target was.
  msg_container.addEventListener("scrollend", _ => {
    update_timeout = setTimeout(() => snapped_message = visible_message, 50);
  });
  msg_container.addEventListener("scroll", _ => {
    if (update_timeout != null) {
      clearTimeout(update_timeout);
      update_timeout = null;
    }
  });
} else {
  msg_container.addEventListener("scroll", _ => {
    if (update_timeout != null) {
      clearTimeout(update_timeout);
      update_timeout = null;
    }
    update_timeout = setTimeout(() => snapped_message = visible_message, 50);
  });
}
// Set up scrolling with left/right arrow keys
document.body.addEventListener("keydown", e => {
  if (e.key === "ArrowLeft") {
    e.preventDefault();
    scroll_messages_by(-1);
  } else if (e.key === "ArrowRight") {
    e.preventDefault();
    scroll_messages_by(1);
  }
});
// Set up intersection observer on messages
const observer_messages = new IntersectionObserver((entries, _) => {
  let intersection_highest = 0;
  for (const e of entries) {
    if (e.isIntersecting && e.intersectionRatio >= 0.5 && e.intersectionRatio > intersection_highest) {
      intersection_highest = e.intersectionRatio;
      visible_message = e.target;
    }
  }
  if (snapped_message == null) {
    snapped_message = visible_message;
  }
}, { root: msg_container, threshold: [0.5, 1.0] });
for (const el of messages) {
  observer_messages.observe(el);
}

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
  if (snapped_message == null) {
    return;
  }
  if (num > 0) {
    for (let i = 0; i < num; i++) {
      if (snapped_message.nextElementSibling == null) {
        break;
      }
      snapped_message = snapped_message.nextElementSibling;
    }
  } else {
    for (let i = 0; i < -num; i++) {
      if (snapped_message.previousElementSibling == null) {
        break;
      }
      snapped_message = snapped_message.previousElementSibling;
    }
  }
  snapped_message.scrollIntoView({ behavior: "smooth", inline: "center" });
}
