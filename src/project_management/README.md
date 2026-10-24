Each project will have a logag.project.toml.
This toml file will contain the project identifier to which an agent connects to.

To connect to the project stream securly:

## Creation:

 - To initiate/associate a folder with a given project stream -> LLM initially requests a project key to use log ag via the auth setup.
 - LLM stores the project key in the file.

## How to ensure that an unauthorized system never connects to the project stream?

 - Key the project lanes on company id and project name -> this adds a basic level of auth.
 - Use public Private key based encryption to determine if the request is comming from a legitimate source.
 - PP keys help in running logag in headless mode as well.

## Usage
 - On booting up logag, LLm will always need to provide the project key.
 - Project key should be provided with a public key.
