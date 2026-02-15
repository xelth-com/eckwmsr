const { Client } = require('pg');
const fs = require('fs');
const path = require('path');

async function main() {
  let schema = fs.readFileSync(path.join(__dirname, 'eckwms_schema_clean.sql'), 'utf8');
  // Remove psql meta-commands and unavailable extensions
  schema = schema.split('\n')
    .filter(line => !line.startsWith('\\'))
    .join('\n')
    .replace(/CREATE EXTENSION.*?;/gs, '')
    .replace(/SET default_table_access_method.*?;/gs, '');

  const client = new Client({
    host: 'localhost',
    port: 5433,
    user: 'postgres',
    database: 'eckwms',
  });

  await client.connect();
  console.log('Connected to eckwms database');

  // Smart split: handle $$ dollar-quoted strings
  const statements = [];
  let current = '';
  let inDollarQuote = false;
  for (const line of schema.split('\n')) {
    current += line + '\n';
    if (line.includes('$$')) {
      const count = (line.match(/\$\$/g) || []).length;
      if (count % 2 === 1) inDollarQuote = !inDollarQuote;
    }
    if (!inDollarQuote && line.trim().endsWith(';')) {
      statements.push(current.trim());
      current = '';
    }
  }
  if (current.trim()) statements.push(current.trim());

  let ok = 0, errors = 0;
  for (const stmt of statements) {
    if (stmt.length < 3) continue;
    try {
      await client.query(stmt);
      ok++;
    } catch (e) {
      errors++;
      if (errors <= 10) {
        const msg = e.message.substring(0, 150);
        // Skip noise
        if (!msg.includes('existiert bereits')) console.log('  Error:', msg);
      }
    }
  }
  console.log(`Schema: ${ok} OK, ${errors} errors (out of ${statements.length})`);

  // Check if user_auths exists
  try {
    const check = await client.query("SELECT count(*) FROM information_schema.tables WHERE table_name = 'user_auths'");
    console.log('user_auths table exists:', check.rows[0].count > 0 ? 'YES' : 'NO');
  } catch (e) {
    console.log('Check failed:', e.message);
  }

  // Fix search_path
  await client.query("SET search_path TO public;");

  // Create user
  try {
    await client.query(`
      INSERT INTO user_auths (id, username, password, email, name, role, user_type, pin, is_active, preferred_language, created_at, updated_at)
      VALUES (
        gen_random_uuid(), 'Dimi', '$2b$10$4QrZRyH1eWO6eaaHKWnv7eDfVcSkRlCn.5Dl7ZaqPfSBkvveZOKmm',
        'd.suro@inbody.com', 'Dmytro Surovtsev', 'admin', 'individual', '', true, 'en', NOW(), NOW()
      ) ON CONFLICT (email) DO NOTHING;
    `);
    console.log('User created');
  } catch (e) {
    console.log('User creation:', e.message.substring(0, 100));
  }

  await client.end();
  console.log('Done');
}

main().catch(e => { console.error(e.message); process.exit(1); });
