const { Storage } = require('@google-cloud/storage');
const { OAuth2Client } = require('google-auth-library');
const CLIENT_ID = "595809100273-9vvaauhe57ovk2s67g6buuqptsiamfrg.apps.googleusercontent.com";
const client = new OAuth2Client(CLIENT_ID);
const uuid = require('uuid');

async function verify(token) {
    const ticket = await client.verifyIdToken({
        idToken: token,
        audience: CLIENT_ID, // or [CLIENT_ID_1, CLIENT_ID_2, ...]
    });
    const payload = ticket.getPayload();
    let user = {};
    user.id = payload['sub'];
    user.name = payload['name'];
    return user;
}

exports.getSession = async (req, res) => {



    res.set('Access-Control-Allow-Origin', '*');

    if (req.method === 'OPTIONS') {
        res.set('Access-Control-Allow-Methods', 'GET, POST');
        res.set('Access-Control-Allow-Headers', 'Content-Type');
        res.set('Access-Control-Max-Age', '86400');
        res.status(204).send('');
    } else {
        const storage = new Storage();

        let session = null;
        if (req.query.sessionId) {
            //get the stored session.
            session = await getSession(storage, req.query.sessionId);
            if (session) {
                //TODO: check expiration date
            }
        }

        if (!session) {
            session = {};
            session.user = await verify(req.body.credential);
            session.sessionId = uuid.v1();
        }
        let expirationTime = 2 * 60 * 60 * 1000;

        session.expirationDate = new Date();
        session.expirationDate.setTime(session.expirationDate.getTime() + expirationTime);
        //create session file.
        await storage
            .bucket("mathparser-session-data")
            .file(session.sessionId)
            .save(JSON.stringify(session));

        res.status(200).json(session);
    }


};

//await-able because returns a Promise
function streamToString(stream) {
    const chunks = [];
    return new Promise((resolve, reject) => {
        stream.on('data', (chunk) => chunks.push(Buffer.from(chunk)));
        stream.on('error', (err) => resolve(null));
        stream.on('end', () => resolve(Buffer.concat(chunks).toString('utf8')));
    })
}

async function getSession(storage, sessionId) {
    let readStream = await storage
        .bucket("mathparser-session-data")
        .file(sessionId)
        .createReadStream();

    let strSession = await streamToString(readStream);

    return JSON.parse(strSession);
}