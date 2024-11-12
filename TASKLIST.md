# Development task list

The orchestrator receives payloads and keeps it in staging.

Each payload is accompanied by some descriptions which contains:

- user id
- service
- access level

The orchestrator makes a decision to send the payload to the service based
on criteria, such as how many jobs of a given user are already running.

It submits the payload to the service and keeps track of its status.

Under the hood each service might have different destinations;

It retrieves the output from the destination and sends it back.

---

- Payload description
- Service configuration
- Endpoints
- Queue logic
- Status tracking
- Submission wrappers

---

## Endpoints

- `/upload` - receives the payload, returns a job identifier
- `/status/<identifier>` - checks the status of this job
- `/download/<identifier>` - downloads the job output

## Payload description

```yaml
# base64 (for now) containing all the payload data
input: "base64file"
# service targeted for it
service: string
# user identification
user_id: <>
```

## Service configuration

...? how to configure this?

```yaml
name:
destinations:
  - jobd: endpoint
  - slurml: endpoint
```
