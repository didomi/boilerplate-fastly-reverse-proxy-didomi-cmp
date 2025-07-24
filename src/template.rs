pub const HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Travel Southeast Asia</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            margin: 0;
            padding: 0;
            background-color: #f4f4f4;
        }
        header {
            background: url('https://picsum.photos/1200/400?random=1') no-repeat center center;
            background-size: cover;
            color: white;
            text-align: center;
            padding: 60px 20px;
        }
        header h1 {
            font-size: 3em;
            margin: 0;
        }
        main {
            display: flex;
            flex-wrap: wrap;
            justify-content: center;
            padding: 20px;
        }
        .location {
            background: white;
            border-radius: 8px;
            box-shadow: 0 4px 8px rgba(0,0,0,0.1);
            margin: 15px;
            overflow: hidden;
            width: 300px;
            transition: transform 0.3s;
        }
        .location:hover {
            transform: translateY(-10px);
        }
        .location img {
            width: 100%;
            height: 200px;
            object-fit: cover;
        }
        .location h2 {
            font-size: 1.5em;
            margin: 15px;
        }
        .location p {
            margin: 0 15px 15px;
            color: #555;
        }
        .ad-container {
            width: 100%;
            text-align: center;
            margin: 30px 0;
        }
    </style>
    
    <!-- Didomi CMP Integration -->
    <script type="text/javascript">(function(){function i(e){if(!window.frames[e]){if(document.body&&document.body.firstChild){var t=document.body;var n=document.createElement("iframe");n.style.display="none";n.name=e;n.title=e;t.insertBefore(n,t.firstChild)}else{setTimeout(function(){i(e)},5)}}}function e(n,o,r,f,s){function e(e,t,n,i){if(typeof n!=="function"){return}if(!window[o]){window[o]=[]}var a=false;if(s){a=s(e,i,n)}if(!a){window[o].push({command:e,version:t,callback:n,parameter:i})}}e.stub=true;e.stubVersion=2;function t(i){if(!window[n]||window[n].stub!==true){return}if(!i.data){return}var a=typeof i.data==="string";var e;try{e=a?JSON.parse(i.data):i.data}catch(t){return}if(e[r]){var o=e[r];window[n](o.command,o.version,function(e,t){var n={};n[f]={returnValue:e,success:t,callId:o.callId};if(i.source){i.source.postMessage(a?JSON.stringify(n):n,"*")}},o.parameter)}}if(typeof window[n]!=="function"){window[n]=e;if(window.addEventListener){window.addEventListener("message",t,false)}else{window.attachEvent("onmessage",t)}}}e("__tcfapi","__tcfapiBuffer","__tcfapiCall","__tcfapiReturn");i("__tcfapiLocator")})();</script><script type="text/javascript">(function(){(function(e,r){var t=document.createElement("link");t.rel="preconnect";t.as="script";var n=document.createElement("link");n.rel="dns-prefetch";n.as="script";var i=document.createElement("script");i.id="spcloader";i.type="text/javascript";i["async"]=true;i.charset="utf-8";var o="https://YOUR_DOMAIN/consent/"+e+"/loader.js?target_type=notice&target="+r;if(window.didomiConfig&&window.didomiConfig.user){var a=window.didomiConfig.user;var c=a.country;var d=a.region;if(c){o=o+"&country="+c;if(d){o=o+"&region="+d}}}t.href="https://YOUR_DOMAIN/consent/";n.href="https://YOUR_DOMAIN/consent/";i.src=o;var s=document.getElementsByTagName("script")[0];s.parentNode.insertBefore(t,s);s.parentNode.insertBefore(n,s);s.parentNode.insertBefore(i,s)})("24cd3901-9da4-4643-96a3-9b1c573b5264","J3nR2TTU")})();</script>
</head>
<body>
    
    <!-- Footer with Didomi preferences button -->
    <footer style="text-align: center; padding: 40px 20px; background: #333; color: white; margin-top: 40px;">
        <h3>Privacy Settings</h3>
        <p>You can change your consent preferences at any time by clicking the button below.</p>
        <button id="didomi-preferences-btn" 
                onclick="if(window.Didomi) { Didomi.preferences.show('purposes'); } else { console.log('Didomi not loaded yet'); }"
                style="background: #4CAF50; color: white; padding: 12px 24px; border: none; border-radius: 6px; cursor: pointer; font-size: 16px; margin: 10px;">
            🔧 Manage Cookie Preferences
        </button>
    </footer>

</body>
</html>"#;