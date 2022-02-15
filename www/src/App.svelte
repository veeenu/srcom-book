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
</script>

<main>
    <div>
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
    {#await promise}
      <p>Loading...</p>
    {:then pending_games}
      {#each pending_games as { pending, game }}
        <h1>{game}</h1>
        <div class="grid">
          <div class="th">Runner</div>
          <div class="th">Run</div>
          <div class="th">Comment</div>
          <div class="th">Booked by</div>
          <div class="th">Book</div>
          {#each pending as run}
            <div>
              <p>
                <span class="runner">
                  <a href="{run.player_url}" target="_blank">
                    {run.player_name} {getFlagEmoji(run.player_location)}
                  </a>
                </span>
                <br/>
                <span class="submitted">
                  <time datetime={run.submitted}>{convertDate(run.submitted)}</time>
                </span>
              </p>
            </div>
            <div class="run">
              <p>
                <a href="{run.weblink}" target="_blank">
                  <span>{run.category} 🔎</span>
                  <br/>
                  <span class="time">
                    {run.times}
                  </span>
                </a>
              </p>
            </div>
            <div class="comment">
              {run.comment}
            </div>
            <div>
              {run.booked_by || ""}
            </div>
            <div>
              {#if run.booked_by == null}
              <button on:click={() => bookRun(run)}>Book</button>
              {:else if run.booked_by == username}
              <button on:click={() => unbookRun(run)}>Unbook</button>
              {/if}
            </div>
          {/each}
        </div>
      {/each}
    {:catch error}
      <p>{error}</p>
    {/await}
</main>

<style>
	main {
    margin: auto;
    padding: 3rem;
    max-width: 800px;
	}

  main > div.grid {
    display: grid;
    align-items: center;
    grid-template-columns: 3fr 3fr 2fr 2fr 1fr;
  }

  main > div.grid > div {
    padding: .5em;
  }

  .th {
    background-color: rgb(3, 136, 87);
    padding: .5em;
  }

  .runner {
    font-size: 1.2em;
  }

  .time {
    font-size: 0.8em;
  }

  .submitted {
    font-size: 0.8em;
  }

  button {
    cursor: pointer;
  }
</style>
