# Yian And StoryLock User Guide

Yian brings important approval requests back to the user's own local device. After downloading the Yian app and completing one binding step, users can review the request source, understand the request content, and decide whether to approve it locally.

Current online entry:

`https://yian.cdao.online`

## The Big Picture

Yian is the cloud entry and local approval entry in the StoryLock flow. It brings external requests to your local device so you can review and approve them locally before anything continues.

At the software-structure level, the system is divided into three parts:

1. Cloud entry: the Yian Remote Access Interface runs on a cloud platform. It provides the guide, download entry, binding entry, external request entry, and request status view.
2. Local assistant: the Yian app you download runs on your own local device and carries the Private Assistant. It receives requests, explains source and risk, and passes approval work to the local core.
3. Local core: StoryLock Local Core handles story files, key isolation, and sensitive approval on the local device. It returns only the minimum necessary result to the Private Assistant.

So the Yian app you download is not the remote cloud service itself. It is the local approval entry that connects Yian Remote Access Interface with StoryLock Local Core. In everyday use, you use the remote access interface to download and bind; when a remote password-fill, signature, or other interactive authorization request arrives, you check the request status and complete approval on your own local device.

![Yian and StoryLock runtime relationship](../../../src/yian-web/public/assets/yian-network-banner.png)

## What You Use

1. Yian Remote Access Interface: download the app, open the binding entry, inspect request status, and check package information.
2. Yian app: installed on your local device to receive binding information and approval requests.
3. Private Assistant: runs on your local device, explains request source, content, and risk, and can help draft story templates.
4. StoryLock Local Core: handles local approval, story storage, and sensitive execution.

A local device can be a phone or another device controlled by you.

## Quick Start

1. Open the Yian Remote Access Interface: `https://yian.cdao.online`.
2. Click "Download Yian App" to download the current package.
3. Before installing, check the displayed version, file size, and checksum.
4. Install the Yian app on your local device.
5. Return to the Yian Remote Access Interface and click "Open Binding Entry".
6. Follow the local device prompts to complete the first binding.
7. Return to the "Request Status" area on the remote access interface and confirm whether the local device can receive requests.

## How Approval Works

After binding, when a request needs local approval:

1. Check the request source on your local device.
2. Read the explanation and risk notice from the Private Assistant.
3. Confirm that the request matches the action you are performing.
4. Follow the local device prompts for unlock, biometrics, or device credential approval.
5. After approval, the result returns to the Private Assistant, which can then sync confirmation status as needed.

If you do not recognize the request source, or the request does not match your current action, do not approve it.

## Check Request Status

In the "Request Status" area of the Yian Remote Access Interface, you can check:

1. Whether any request is waiting for action.
2. Where the request comes from and what it asks you to do.
3. Whether the request is pending, approved, canceled, or completed.
4. Whether the local device can receive new requests.
5. Current package version, size, and checksum.

If the local device cannot receive requests, open the Yian app, check the local device network, and then refresh request status on the remote access interface. If it still does not recover, open the binding entry and bind again.

## Safety Notes

1. Download the app only from a Yian page you trust.
2. Do not install packages from unknown links, forwarded messages, or unclear sources.
3. Before installing, check the version, file size, and SHA-256 checksum.
4. Do not forward the binding link to other people.
5. Keep your local device network available so the Private Assistant can receive requests and return confirmation status.
6. If a request looks unfamiliar, leave it unapproved for the moment and check the source.

## FAQ

### What if request status does not update?

Confirm that the local device network is available, open the Yian app, wait a few seconds, and then check request status again. If it still does not update, bind again.

### What device should the binding link connect to?

The binding link connects the current environment to a trusted local device. The key question is not who owns the device, but whether you trust and control it and intend to let it receive and handle future requests.

### What if the system warns me during installation?

Install only from a trusted Yian page. Before installing, check the version, file size, and SHA-256 checksum. If the source is unclear, do not install it.

### What should I do after changing local devices?

First export or back up the local files from the old device, then upload or move those files to the new device. After confirming the files are available on the new device, install the Yian app again, bind from the Yian Remote Access Interface, and check request status. Stop using the old connection when the old device is no longer in use.

## Maintenance Note

Historical English design analysis files have been archived under `docs/management/back`. The current `docs/design/en` directory keeps the active English user guide and design documents.
