const net = require('net');
// bcrypt hash for 'admin123' - $2b$10$4QrZRyH1eWO6eaaHKWnv7eDfVcSkRlCn.5Dl7ZaqPfSBkvveZOKmm
const sql = `
INSERT INTO user_auths (id, username, password, email, name, role, user_type, pin, is_active, preferred_language, created_at, updated_at)
VALUES (
  gen_random_uuid(), 'Dimi', '$2b$10$4QrZRyH1eWO6eaaHKWnv7eDfVcSkRlCn.5Dl7ZaqPfSBkvveZOKmm',
  'd.suro@inbody.com', 'Dmytro Surovtsev', 'admin', 'individual', '', true, 'en', NOW(), NOW()
) ON CONFLICT (email) DO NOTHING;
`;

const client = new net.Socket();
let authDone = false;

client.connect(5433, '127.0.0.1', () => {
  const buf = Buffer.alloc(1024);
  let offset = 4;
  buf.writeInt32BE(196608, offset); offset += 4;
  offset += buf.write('user\0', offset);
  offset += buf.write('postgres\0', offset);
  offset += buf.write('database\0', offset);
  offset += buf.write('eckwms\0', offset);
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
    const q = sql;
    const qbuf = Buffer.alloc(q.length + 6);
    qbuf[0] = 81;
    qbuf.writeInt32BE(q.length + 5, 1);
    qbuf.write(q + '\0', 5);
    client.write(qbuf);
    console.log('User creation SQL sent');
  }
  if (client._sent && str.includes('Z')) {
    console.log('Done');
    client.end();
    process.exit(0);
  }
});
client.on('error', () => {});
setTimeout(() => { console.log('Timeout'); process.exit(0); }, 5000);
