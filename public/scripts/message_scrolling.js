const msg_container = document.getElementById("msg-container");
/** @type {Element[]} */
let visible_messages = [];
/** @type {Element | null} */
let current_message;
let update_timeout;

// Work around Safari not having scrollend event
if ("onscrollend" in msg_container) {
  // We don't want to update the index immediately on scrollend because with smooth scrolling,
  // consecutive clicks of the buttons or arrow keys will actually stop the current scrolling
  // and then start scrolling again (at least on Chrome), thus firing a scrollend event when we
  // want to keep the index on whatever the previous target was.
  msg_container.addEventListener("scrollend", _ => {
    update_timeout = setTimeout(() => {
      current_message = get_current_message();
      update_timeout = null;
    }, 50);
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
    update_timeout = setTimeout(() => {
      current_message = get_current_message();
      update_timeout = null;
    }, 50);
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
// Set up intersection observer on messages to track which ones are within the visible area
const observer_messages = new IntersectionObserver((entries, _) => {
  for (const e of entries) {
    if (e.isIntersecting) {
      visible_messages.push(e.target);
    } else {
      const index_remove_message = visible_messages.indexOf(e.target);
      if (index_remove_message >= 0) {
        visible_messages.splice(index_remove_message, 1);
      }
    }
  }
  if (current_message == null) {
    current_message = get_current_message();
  }
}, { root: msg_container, threshold: 0 });
for (const el of msg_container.children) {
  observer_messages.observe(el);
}

function get_current_message() {
  const center_view = document.documentElement.clientWidth / 2;
  let dist_center_closest = Number.POSITIVE_INFINITY;
  let candidate_message = null;
  for (const el of visible_messages) {
    const center_message = el.getBoundingClientRect().left + el.clientWidth / 2;
    const dist_center = Math.abs(center_message - center_view);
    if (dist_center < dist_center_closest) {
      candidate_message = el;
      dist_center_closest = dist_center;
    }
  }
  return candidate_message;
}

/**
 * @param num {number}
 */
function scroll_messages_by(num) {
  if (current_message == null) {
    return;
  }
  if (num > 0) {
    for (let i = 0; i < num; i++) {
      if (current_message.nextElementSibling == null) {
        break;
      }
      current_message = current_message.nextElementSibling;
    }
  } else {
    for (let i = 0; i < -num; i++) {
      if (current_message.previousElementSibling == null) {
        break;
      }
      current_message = current_message.previousElementSibling;
    }
  }
  current_message.scrollIntoView({ behavior: "smooth", inline: "center" });
}
