const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const roomName = document.getElementById('room-id');

const entranceDialog = new class {
  /** @type {HTMLDialogElement} */
  #dialog = document.getElementById('entrance');
  #room = document.getElementById('room-id-input');
  #roomIds = document.getElementById('room-ids');
  #name = document.getElementById('name-input');
  #enter = document.getElementById('enter');

  constructor() {
    this.#name.onkeyup = _ => {
      if (this.#name.value.trim().length) {
        this.#enter.disabled = false;
      } else {
        this.#enter.disabled = true;
      }
    };

    this.#enter.onclick = async e => {
      const roomId = this.#room.value.trim() || this.#room.placeholder;
      userName = this.#name.value.trim();
      if (!this.#name.checkValidity()) {
        this.#name.reportValidity();
        return;
      }
      await enterRoom(roomId, userName);
      roomName.textContent = roomId;
      this.close();
    };
  }

  show() {
    this.#dialog.showModal();
  }

  close() {
    this.#dialog.close();
  }

  setRoomIds(ids) {
    for (const id of ids) {
      this.#roomIds.insertAdjacentHTML('beforeend', `<option>${id}</option>`);
    }
  }
}();

const messageContainer = new class {
  #container = document.getElementById('messages');

  append(message) {
    const date = new Date();
    const datetime = `${date.getMonth() + 1}/${date.getDate()} ${date.getHours()}:${date.getMinutes()}`;
    const html = this.#createElement(message.name, message.message, datetime);
    this.#container.insertAdjacentHTML('afterbegin', html);
    this.#purge();
  }

  #purge() {
    const messages = Array.from(this.#container.getElementsByClassName('message'));
    if (messages.length >= 100) {
      messages.slice(100).forEach(el => el.remove());
    }
  }

  #createElement(name, message, datetime) {
    return `<div class="message">
      <div class="message-head">
        <label class="message-name">${name}</label>
        <div class="message-time">${datetime}</div>
      </div>
      <div class="message-body">${message}</div>
    </div>`;
  }
}();

const messageInput = new class {
  /** @type {HTMLInputElement} */
  #input = document.getElementById('message');
  #send = document.getElementById('send');

  constructor() {
    this.#send.onclick = e => {
      const message = this.#input.value.trim();
      if (message.length > 0) {
        sendMessage(message);
        this.#input.value = '';
      }
    };
  }
}();

const noticeDialog = new class {
  /** @type {HTMLDialogElement} */
  #dialog = document.getElementById('notice');
  #message = document.getElementById('notice-message');
  #close = document.getElementById('notice-close');

  constructor() {
    this.#close.onclick = _ => this.#dialog.close();
  }

  show(message) {
    this.#message.textContent = message;
    this.#dialog.showModal();
  }
}();

let userName = '';

async function sendMessage(message) {
  await invoke('send_message', {
    name: userName,
    message
  });
  messageInput.value = '';
}

async function enterRoom(room, name) {
  invoke('enter_room', {
    room,
    name
  });
}


listen('message', event => {
  const message = event.payload;
  messageContainer.append(message);
});

listen('notice', event => {
  const { message } = event.payload;
  noticeDialog.show(message);
});

entranceDialog.show();
