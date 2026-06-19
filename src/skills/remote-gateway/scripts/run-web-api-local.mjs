import { createServer } from 'node:http';
import gatewayHandler from '../../../../web-api/storylock-gateway.mjs';

const host = process.env.HOST || '127.0.0.1';
const port = Number(process.env.PORT || 4318);

const server = createServer((req, res) => gatewayHandler(req, res));

server.listen(port, host, () => {
  console.log(`StoryLock Web API gateway running at http://${host}:${port}/`);
});
