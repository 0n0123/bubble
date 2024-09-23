const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;

const roomIdList = document.querySelector('#room-ids');
const roomName = document.querySelector('#room-id');
const entranceDialog = document.querySelector('dialog#entrance');
const roomIdInput = document.querySelector('input#room-id-input');
const nameInput = document.querySelector('input#name-input');
const enterButton = document.querySelector('button#enter');
const messageContainer = document.querySelector('div#messages');
const messageInput = document.querySelector('input#message');
const sendButton = document.querySelector('button#send');

let userName = '';

async function sendMessage(message) {
  await invoke('send_message', { 
    name: userName,
    message
  });
  messageInput.value = '';
}

window.addEventListener('DOMContentLoaded', async () => {
  await invoke('hello', {});

  entranceDialog.show();
  enterButton.onclick = async e => {
    userName = nameInput.value.trim();
    const roomId = roomIdInput.value.trim();

    await enterRoom(roomId, userName);

    roomName.textContent = roomId;
    entranceDialog.hide();
  };

  sendButton.onclick = e => {
    const message = messageInput.value.trim();
    sendMessage(message);
  };
});

async function enterRoom(room, name) {
  invoke('enter_room', {
    room,
    name
  });
}

listen('rooms', event => {
  const roomIds = event.payload;
  for (const roomId of roomIds) {
    roomIdList.insertAdjacentHTML('beforeend', `<option value="${roomId}"></option>`);
  }
});

listen('message', event => {
  const message = event.payload;
  const date = new Date();
  const datetime = `${date.getMonth() + 1}/${date.getDate()} ${date.getHours()}:${date.getMinutes()}`;
  const html = createMessageHTML(message.name, message.message, datetime);
  messageContainer.insertAdjacentHTML('afterbegin', html);
  purgeMessage();
});

function createMessageHTML(name, message, datetime) {
  return `<div class="message">
    <div class="message-head">
      <label class="message-name">${name}</label>
      <div class="message-time">${datetime}</div>
    </div>
    <div class="message-body">${message}</div>
  </div>`;
}

function purgeMessage() {
  const messages = Array.from(document.querySelectorAll('.message'));
  if (messages.length >= 100) {
    messages.slice(100).forEach(el => el.remove());
  }
}