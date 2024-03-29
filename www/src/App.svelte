<script>
  const { API_ROOT } = ENV
  let username
  let api_key = localStorage.getItem('api-key')
  let promise = getPending()
  let auth_promise = checkCredentials()

  async function getPending() {
      let games = await(await fetch(`${API_ROOT}/games`)).json()
      let pending = await(await fetch(`${API_ROOT}/pending`)).json()

      const ordering = [
        "nd28z0ed",
        "w6jve26j",
        "lde3woe6",
        "m1zky010",
        "y65lw01e",
        "k6qg0xdg",
        "9d3kqg1l",
        "m1mn8kd2",
        "j1neogy1",
        "o1y9zk26",
      ]
      
      let r = ordering.map((id) => ({ game: games[id], pending: pending[id] }))
      return r
  }

  async function updatePending() {
      let pending = await getPending()

      promise = new Promise((res) => res(pending))
    }

  function getFlagEmoji(countryCode) {
    if (!countryCode) {
      return ''
    }

    const codePoints = countryCode
      .toUpperCase()
      .split('')
      .map(char =>  127397 + char.charCodeAt());
    return String.fromCodePoint(...codePoints);
  }

  function convertDate(date) {
    let dateobj = new Date(date)
    let [y, m, d] = [dateobj.getFullYear(), dateobj.getMonth() + 1, dateobj.getDate()]

    m = m.toString().padStart(2, '0')
    d = d.toString().padStart(2, '0')

    return `${y}-${m}-${d}`
  }

  async function copyRunUrl(run) {
    await navigator.clipboard.writeText(run.video_url)
  }

  async function bookRun(run) {
    console.log(`Booking run ${run.id}`)

    await fetch(`${API_ROOT}/book/${run.id}`, {
        method: 'POST',
        headers: {
            'X-API-Key': api_key
        }
    })

    await updatePending()
  }

  async function unbookRun(run) {
    console.log(`Unbooking run ${run.id}`)

    await fetch(`${API_ROOT}/book/${run.id}`, {
        method: 'DELETE',
        headers: {
            'X-API-Key': api_key
        }
    })

    await updatePending()
  }

  async function checkCredentials() {
    let resp = await fetch(`${API_ROOT}/auth`, {
        headers: {
            'X-API-Key': api_key
        }
    })

    if (resp.status == 200) {
        username = await resp.text()
        console.log(username)
        return true
    } else {
        username = null
        return false
    }
  }

  function storeCredentials() {
    auth_promise = checkCredentials()
    localStorage.setItem('api-key', api_key)
  }

  function abridged(s) {
      if (s.length < 10) {
          return s
      } else {
          return s.substring(0, 10) + '…'
      }
  }
</script>

<main>
  {#await promise}
    <p>Loading...</p>
  {:then pending_games}
    <div id="sidebar">
      <div id="api-key">
        <input type="text" placeholder="Paste your API key here" bind:value={api_key} on:change={storeCredentials}/>
        {#await auth_promise}
          ...
        {:then outcome}
          {#if outcome}
          ✅
          <span>{username}</span>
          {:else}
          ❌
          <a href="https://www.speedrun.com/api/auth" target="_blank">Copy your API Key from here</a>
          {/if}
        {:catch err}
          ❌
          <a href="https://www.speedrun.com/api/auth" target="_blank">Copy your API Key from here</a>
        {/await}
      </div>
    </div>
    <div id="games">
      <div>
      {#each pending_games as { pending, game }}
        <input class="game-toggle" type="checkbox" id={game} checked/>
        <label class="game-toggle" for={game}>
          <h1>{game}</h1>
        </label>
        <div class="grid">
          <div class="th runner-th">Runner</div>
          <div class="th run-th">Run</div>
          <div class="th book-th">Book</div>
          {#each pending as run}
            <div class="run">
              <p>
                <a href="{run.player_url}" target="_blank">
                  <span class='emoji'>{getFlagEmoji(run.player_location)}</span>
                  <span class="runner">{run.player_name}</span>
                </a>
                <br/>
                <span class='emoji'>📅</span>
                <span class="submitted">
                  <time datetime={run.submitted}>{convertDate(run.submitted)}</time>
                </span>
                <br/>
                {#if run.comment.length > 0}
                <label class='comment'>
                  <input type='checkbox'/>
                  <span class='emoji'>💬</span>
                  <span class='comment-abridged'>{abridged(run.comment)}</span>
                  <span class='comment-full'>{run.comment}</span>
                </label>
                {/if}
              </p>
            </div>
            <div class="run">
              <a href="{run.weblink}" target="_blank">
                <p>
                  <span class='emoji'>🔎</span>
                  <span class='category'> {run.category}</span>
                  <br/>
                  <span class='emoji'>⏱️ </span>
                  <span class="time"> {run.times}</span>
                </p>
              </a>
              <p>
                <label class="web-url">
                  <input type="text" value={run.video_url}/>
                  <button on:click={async () => await copyRunUrl(run) }>🔗</button>
                </label>
              </p>
            </div>
            <div class="book">
              {#if run.booked_by == null}
                <button on:click={() => bookRun(run)}>Book</button>
              {:else}
                {run.booked_by || ""}
                {#if  run.booked_by == username}
                  <br/>
                  <button on:click={() => unbookRun(run)}>Unbook</button>
                {/if}
              {/if}
            </div>
          {/each}
        </div>
      {/each}
      </div>
    </div>
  {:catch error}
    <p>{error}</p>
  {/await}
</main>

<style>
	main {
    margin: auto;
    padding: 3rem;
    width: 540px;
	}

  div.grid {
    width: 540px;
    display: grid;
    align-items: center;
    grid-template-columns: 3fr 3fr 2fr;
  }

  label.comment > input {
    display: none;
  }

  .comment-abridged {
    display: none;
  }

  @media screen and (max-width: 900px) {
    div.grid {
      grid-template-columns: 3fr 3fr 2fr;
    }
  }

  label.game-toggle {
    cursor: pointer;
    user-select: none;
  }

  input.game-toggle {
    display: none;
  }

  input.game-toggle:checked + label.game-toggle + div.grid {
    display: grid;
  }

  input.game-toggle:not(:checked) + label.game-toggle + div.grid {
    display: none;
  }

  input.game-toggle:checked + label.game-toggle > h1::before {
    content: "\25bc\00a0";
  }

  input.game-toggle:not(:checked) + label.game-toggle > h1::before {
    content: "\25b6\00a0";
  }

  label.comment > input:checked ~ .comment-abridged,
  label.comment > input:not(:checked) ~ .comment-full {
    display: none;
  }

  label.comment > input:checked ~ .comment-full,
  label.comment > input:not(:checked) ~ .comment-abridged {
    display: inline;
  }

  div.grid > div {
    padding: .5em;
  }

  span.emoji {
    font-size: 1rem;
  }

  div.book {
    text-align: center;
  }

  .th {
    background-color: rgb(3, 136, 87);
    padding: .5em;
  }

  .runner {
    font-size: 1.2em;
  }

  .category,
  .time,
  .submitted,
  .comment-abridged,
  .comment-full {
    font-size: 0.9em;
  }

  label.comment {
    user-select: none;
  }

  button {
    cursor: pointer;
  }

  div.run > a > p {
    margin-bottom: 0;
  }

  div.run > p {
    margin-top: 0;
  }

  label.web-url {
    display: grid;
    grid-template-columns: auto 2em;
    border: 1px solid rgba(255, 255, 255, .3);
  }

  label.web-url > * {
    font-family: monospace;
    font-size: .7rem;
    border: none;
    padding-bottom: 0;
  }

  label.web-url > *:focus-visible {
    outline: none;
  }
</style>
