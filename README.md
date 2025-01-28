# Marvel Rivals Mod Manager DB

## Description

Marvel Rivals Mod Manager DB is a RESTful api and database that allows for a faster and more seamless way to download mods for the people on my network using my mod manager

## TODO

- Better error handling

## Base URL

```
http://localhost:8080
```

## Endpoints

### 1. Setup Database

#### **GET** `/setup`

Sets up the database by creating the necessary tables.

**Response:**

- **Status Code:** `200 OK` if the setup is successful.

### 2. Upload Mod

#### **POST** `/upload`

Uploads a mod with metadata and files.

**Request:**

- **Form Data:**
    - `id` (text): The unique identifier for the mod.
    - `title` (text): The title of the mod.
    - `version` (text): The version of the mod.
    - `thumbnail` (file): The thumbnail image file for the mod.
    - `file` (file): The mod file.

**Response:**

- **Status Code:** `200 OK` if the upload is successful.

### 3. Get Metadata

#### **GET** `/metadata`

Retrieves metadata for all mods.

**Response:**

- **Status Code:** `200 OK`
- **Body:** JSON array of mod metadata objects.
  ```json
  [
    {
      "id": "string",
      "title": "string",
      "version": "string",
      "thumbnail": "base64 string",
      "file_path": "string"
    }
  ]
  ```

### 4. Download Mod

#### **GET** `/download/{id}`

Downloads the mod file with the specified ID.

**Path Parameters:**

- `id` (string): The unique identifier for the mod.

**Response:**

- **Status Code:** `200 OK` if the download is successful.
- **Body:** The mod file.

## Data Models

### Mod Metadata

```json
{
  "id": "string",
  "title": "string",
  "version": "string",
  "thumbnail": "base64 string",
  "file_path": "string"
}
```

## Error Handling

### Common Error Responses

- 400 Bad Request: Invalid request data.
- 404 Not Found: Resource not found.
- 500 Internal Server Error: Server encountered an error.
