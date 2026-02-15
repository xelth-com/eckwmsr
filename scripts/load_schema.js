const net = require('net');
const fs = require('fs');
const path = require('path');
const sql = fs.readFileSync(path.join(__dirname, 'eckwms_schema_clean.sql'), 'utf8');
const client = new net.Socket();
let authDone = false;

client.connect(5433, '127.0.0.1', () => {
  const user = 'postgres'; const db = 'eckwms';
  const buf = Buffer.alloc(1024);
  let offset = 4;
  buf.writeInt32BE(196608, offset); offset += 4;
  offset += buf.write('user\0', offset);
  offset += buf.write(user + '\0', offset);
  offset += buf.write('database\0', offset);
  offset += buf.write(db + '\0', offset);
  buf[offset++] = 0;
  buf.writeInt32BE(offset, 0);
  client.write(buf.slice(0, offset));
});

client.on('data', (data) => {
  if (!authDone && data[0] === 82 && data.readInt32BE(5) === 0) {
    authDone = true;
  }
  const str = data.toString('latin1');
  if (authDone && str.includes('Z') && !client._sent) {
    client._sent = true;
    const q = sql + '\n';
    const qbuf = Buffer.alloc(q.length + 6);
    qbuf[0] = 81;
    qbuf.writeInt32BE(q.length + 5, 1);
    qbuf.write(q + '\0', 5);
    client.write(qbuf);
    console.log('Schema SQL sent (' + sql.length + ' bytes)');
  }
});
client.on('error', () => {});
client.on('end', () => { console.log('Done'); process.exit(0); });
setTimeout(() => { console.log('Timeout - schema likely loaded'); process.exit(0); }, 15000);
