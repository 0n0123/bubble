const { invoke } = window.__TAURI__.tauri;

const roomName = document.querySelector('#room-id');
const entranceDialog = document.querySelector('dialog#entrance');
const roomIdInput = document.querySelector('input#room-id-input');
const nameInput = document.querySelector('input#name-input');
const enterButton = document.querySelector('button#enter');
const messageInput = document.querySelector('input#message');
const sendButton = document.querySelector('button#send');

const info = {
  room: '',
  name: '',
};

async function sendMessage(message) {
  await invoke("send_message", { 
    name: info.name,
    room: info.room,
    message
  });
}

window.addEventListener("DOMContentLoaded", () => {
  //entranceDialog.show();
  enterButton.onclick = e => {
    info.room = roomIdInput.value.trim();
    info.name = nameInput.value.trim();
    roomName.textContent = info.room;
    //entranceDialog.hide();
    entranceDialog.style.display = 'none';
  };

  sendButton.onclick = e => {
    const message = messageInput.value.trim();
    sendMessage(message);
  };
});
