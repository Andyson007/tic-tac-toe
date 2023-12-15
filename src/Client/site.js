const ws = new WebSocket("ws://127.0.0.1:9001");

const board = document.getElementById("board");
const size = 3;

ws.addEventListener("message", (event) => {
  let data = event.data;
  console.log(data);
  if (data[0] === ",") {
    if (["2", "3"].includes(data[1])) {
      const str = data[1] === "2" ? "X" : "O";
      alert(`${str} won`)
      data = data.split("\n");
      data.splice(0, 1);
      data = data.join("\n");
    } else {
      console.log(data);
      return;
    }
  }
  const table = [];
  for (const c of data.split("\n")) {
    let tmp = c.split(",");
    tmp.pop()
    tmp = tmp.map((elem) => {
      if (elem === "2") return "X";
      if (elem === "3") return "O";
      return elem;
    })
    table.push(tmp);
  }
  board.innerHTML = "";
  createTable(board, table);
});


function createTable(body, data) {
  // console.log(data);
  const tbl = document.createElement('table');

  for (let i = 0; i < size; i++) {
    const tr = tbl.insertRow();
    for (let j = 0; j < size; j++) {
      const td = tr.insertCell();
      let value = data[i][j];
      if (!(value === "X" || value === "O"))
        value = "";
      td.appendChild(document.createTextNode(value));
      td.addEventListener("click", () => clickEvent(i, j));
      console.log(data[i][j]);
      switch (data[i][j]) {
        case "0": td.classList.add("unknown"); break;
        case "1": td.classList.add("lost"); break;
        case "4": td.classList.add("draw"); break;
        case "5": td.classList.add("won"); break;
      }
    }
  }
  body.appendChild(tbl);
}

createTable(board, [[0, 0, 0], [0, 0, 0], [0, 0, 0]]);

function clickEvent(x, y) {
  ws.send(`${x} ${y}`)
}
