import React, { useState } from "react";
import {binToBase64Url} from "./functions/AuthUtility";
import {computeCodeVerifier, computeRandom, encodedHashBin} from "./functions/OAuth";

export const redirect_uri = "http://localhost:3072/callback"

const AuthRedirect = () => {
    const handleRedirect = async () => {

        //OAuth Authorization Code Flow + PKCE step 1
        const state = binToBase64Url(crypto.getRandomValues(new Uint8Array(16)))
        const { verifier, challenge } = await computeCodeVerifier()
        //OpenID nonce
        const { encoded_bin: nonce_original, random_bin: nonce_bin } = computeRandom()
        const nonce = await encodedHashBin(nonce_bin)

        const params = new URLSearchParams({
            "response_type": "code",
            "client_id":  "reminders.tipten.nl",
            "redirect_uri":  redirect_uri,
            "state": state,
            "code_challenge": challenge,
            "code_challenge_method": "S256",
            "nonce": nonce,
        }).toString()

        // const res = await fetch(`http://localhost:4243/oauth/start/`, {
        //     method: 'POST', body: JSON.stringify(reqst),
        //     headers: {
        //         'Content-Type': 'application/json'
        //     }
        // })
        // const auth_id = await res.text()

        const state_verifier = {
            code_verifier: verifier,
            state
        }
        localStorage.setItem("state_verify", JSON.stringify(state_verifier))
        localStorage.setItem("nonce_original", nonce_original)

        const redirectUrl = "http://localhost:3073/oauth/authorize?" + params

        window.location.replace(redirectUrl);
    }

    return (
        <>
            <div>
                {handleRedirect()}
            </div>
        </>
    )
}

export default AuthRedirect;