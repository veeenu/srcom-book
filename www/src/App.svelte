<script>
  const { API_ROOT } = ENV
  let promise = getPending()
  let username = localStorage.getItem('username')
  let password = localStorage.getItem('password')
  let auth_promise = checkCredentials()

  async function getPending() {
      let pending = await(await fetch(`${API_ROOT}/pending`)).json()
      let mods = await(await fetch(`${API_ROOT}/mods`)).json()

      return [pending, mods]
  }

  async function fetchFromSrcom() {
    await fetch(`${API_ROOT}/fetch`)
    promise = getPending()
  }

  function getFlagEmoji(countryCode) {
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
    let authorization = btoa(`${username}:${password}`)

    await fetch(`${API_ROOT}/book/${run.id}`, {
        method: 'POST',
        headers: {
            'Authorization': `Basic ${authorization}`
        }
    })

    promise = getPending()
  }

  async function unbookRun(run) {
    console.log(`Unbooking run ${run.id}`)
    let authorization = btoa(`${username}:${password}`)

    await fetch(`${API_ROOT}/book/${run.id}`, {
        method: 'DELETE',
        headers: {
            'Authorization': `Basic ${authorization}`
        }
    })

    promise = getPending()
  }

  async function checkCredentials() {
    let authorization = btoa(`${username}:${password}`)
    let res = await fetch(`${API_ROOT}/auth`, {
      headers: {
          'Authorization': `Basic ${authorization}`
      }
    })
    if (res.status == 200) {
      return true 
    }
    else {
      return false
    }
  }

  function storeCredentials() {
    auth_promise = checkCredentials()
    localStorage.setItem('username', username)
    localStorage.setItem('password', password)
  }
</script>

<main>
    <div>
      <input type="text" placeholder="Username" bind:value={username} on:change={storeCredentials}/>
      <input type="password" placeholder="Password" bind:value={password} on:change={storeCredentials}/>
      {#await auth_promise}
        ...
      {:then outcome}
        {#if outcome}
        ✅
        {:else}
        ❌
        {/if}
      {:catch err}
        ❌
      {/await}
    </div>
    {#await promise}
      <p>Loading...</p>
    {:then [pending, mods]}
      <div class="grid">
        <div class="th">Run</div>
        <div class="th">Comment</div>
        <div class="th">Booked by</div>
        <div class="th">Book</div>
        {#each pending as run}
          <div>
            <span class="runner">
              <a href="{run.player_url}" target="_blank">{run.player_name} {getFlagEmoji(run.player_location)}</a>
            </span>
            <br/>
            <span class="time">
              <a href="{run.weblink}" target="_blank">{run.times} 🔎</a>
            </span>
            <br/>
            <span class="submitted">
              <time datetime={run.submitted}>{convertDate(run.submitted)}</time>
            </span>
          </div>
          <div class="comment">{run.comment}</div>
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
    grid-template-columns: 2fr 4fr 2fr 1fr;
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
    font-size: 1em;
  }

  .submitted {
    font-size: 0.8em;
  }

  .run {
    text-align: center;
  }
</style>
